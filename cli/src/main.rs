//! fPOW CLI - Algorand Client
//!
//! Command-line interface for interacting with the fPOW smart contract on Algorand.
//! Uses Nodely's free Algorand API endpoints with built-in rate limiting.
//!
//! ## Nodely Free Tier Limits
//! - 200,000 requests per day (6M per month)
//! - 1000 requests/second shared across all free tier users
//! - 50-150ms artificial delay on responses
//!
//! This CLI implements conservative rate limiting to avoid hitting these limits.

use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use algonaut::algod::v2::Algod;
use algonaut::core::{Address, MicroAlgos};
use algonaut::transaction::{
    account::Account,
    builder::CallApplication,
    TxnBuilder,
};
use anyhow::Result;
use fpow_api::prelude::*;
use tokio::sync::Mutex;
use tokio::time::sleep;

// =============================================================================
// Nodely Endpoints (Free Tier)
// Documentation: https://nodely.io/docs/free/start/
// =============================================================================

/// Nodely mainnet algod API endpoint
pub const NODELY_MAINNET_ALGOD: &str = "https://mainnet-api.4160.nodely.dev";

/// Nodely testnet algod API endpoint
pub const NODELY_TESTNET_ALGOD: &str = "https://testnet-api.4160.nodely.dev";

/// Nodely mainnet indexer API endpoint
pub const NODELY_MAINNET_INDEXER: &str = "https://mainnet-idx.4160.nodely.dev";

/// Nodely testnet indexer API endpoint
pub const NODELY_TESTNET_INDEXER: &str = "https://testnet-idx.4160.nodely.dev";

// =============================================================================
// Rate Limiting Configuration
// Based on Nodely free tier limits: https://nodely.io/docs/free/policy/
// =============================================================================

/// Maximum requests per second (conservative limit, actual shared limit is 1000)
const MAX_REQUESTS_PER_SECOND: u32 = 10;

/// Minimum delay between requests in milliseconds
const MIN_REQUEST_DELAY_MS: u64 = 100; // 10 requests/sec max

/// Maximum daily requests (Nodely free tier limit is 200K)
const MAX_DAILY_REQUESTS: u64 = 180_000; // Leave 10% buffer

/// Initial backoff delay for rate limit errors (429)
const INITIAL_BACKOFF_MS: u64 = 1000;

/// Maximum backoff delay
const MAX_BACKOFF_MS: u64 = 60_000;

/// Maximum retry attempts for rate-limited requests
const MAX_RETRIES: u32 = 5;

// =============================================================================
// Rate Limiter Implementation
// =============================================================================

/// Rate limiter for Nodely API requests
pub struct RateLimiter {
    /// Last request timestamp
    last_request: Mutex<Instant>,
    /// Request count for today
    daily_requests: AtomicU64,
    /// Day start timestamp
    day_start: Mutex<Instant>,
    /// Current backoff delay (increases on rate limit errors)
    current_backoff: AtomicU64,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            last_request: Mutex::new(Instant::now() - Duration::from_secs(1)),
            daily_requests: AtomicU64::new(0),
            day_start: Mutex::new(Instant::now()),
            current_backoff: AtomicU64::new(0),
        }
    }

    /// Wait for rate limit and track request
    pub async fn wait_for_request(&self) -> Result<()> {
        // Check and reset daily counter if needed
        {
            let mut day_start = self.day_start.lock().await;
            if day_start.elapsed() > Duration::from_secs(86400) {
                *day_start = Instant::now();
                self.daily_requests.store(0, Ordering::Relaxed);
                println!("Daily request counter reset");
            }
        }

        // Check daily limit
        let daily_count = self.daily_requests.load(Ordering::Relaxed);
        if daily_count >= MAX_DAILY_REQUESTS {
            return Err(anyhow::anyhow!(
                "Daily request limit reached ({}/{}). Please wait until tomorrow.",
                daily_count, MAX_DAILY_REQUESTS
            ));
        }

        // Apply backoff if we recently hit a rate limit
        let backoff = self.current_backoff.load(Ordering::Relaxed);
        if backoff > 0 {
            println!("Applying backoff delay: {}ms", backoff);
            sleep(Duration::from_millis(backoff)).await;
        }

        // Ensure minimum delay between requests
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        let min_delay = Duration::from_millis(MIN_REQUEST_DELAY_MS);

        if elapsed < min_delay {
            let wait_time = min_delay - elapsed;
            sleep(wait_time).await;
        }

        // Update tracking
        *last = Instant::now();
        self.daily_requests.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Handle rate limit error (429)
    pub fn on_rate_limit(&self) {
        let current = self.current_backoff.load(Ordering::Relaxed);
        let new_backoff = if current == 0 {
            INITIAL_BACKOFF_MS
        } else {
            (current * 2).min(MAX_BACKOFF_MS)
        };
        self.current_backoff.store(new_backoff, Ordering::Relaxed);
        println!("Rate limited! Increasing backoff to {}ms", new_backoff);
    }

    /// Handle successful request
    pub fn on_success(&self) {
        // Gradually reduce backoff on success
        let current = self.current_backoff.load(Ordering::Relaxed);
        if current > 0 {
            let new_backoff = current / 2;
            self.current_backoff.store(new_backoff, Ordering::Relaxed);
        }
    }

    /// Get current stats
    pub fn stats(&self) -> (u64, u64) {
        let daily = self.daily_requests.load(Ordering::Relaxed);
        let backoff = self.current_backoff.load(Ordering::Relaxed);
        (daily, backoff)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// Algorand Client with Rate Limiting
// =============================================================================

/// Algorand client wrapper with built-in rate limiting
pub struct AlgoClient {
    algod: Algod,
    rate_limiter: Arc<RateLimiter>,
    app_id: u64,
}

impl AlgoClient {
    pub fn new(algod_url: &str, algod_token: &str, app_id: u64) -> Result<Self> {
        let algod = Algod::new(algod_url, algod_token)?;
        Ok(Self {
            algod,
            rate_limiter: Arc::new(RateLimiter::new()),
            app_id,
        })
    }

    /// Execute a request with rate limiting and retry logic
    async fn with_rate_limit<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retries = 0;

        loop {
            // Wait for rate limit
            self.rate_limiter.wait_for_request().await?;

            // Execute operation
            match operation().await {
                Ok(result) => {
                    self.rate_limiter.on_success();
                    return Ok(result);
                }
                Err(e) => {
                    let error_str = e.to_string().to_lowercase();

                    // Check if it's a rate limit error
                    if error_str.contains("429") || error_str.contains("too many requests") || error_str.contains("rate limit") {
                        self.rate_limiter.on_rate_limit();
                        retries += 1;

                        if retries >= MAX_RETRIES {
                            return Err(anyhow::anyhow!(
                                "Max retries ({}) exceeded for rate-limited request",
                                MAX_RETRIES
                            ));
                        }

                        println!("Retry {}/{} after rate limit...", retries, MAX_RETRIES);
                        continue;
                    }

                    // Not a rate limit error, return immediately
                    return Err(e);
                }
            }
        }
    }

    /// Get node status
    pub async fn status(&self) -> Result<algonaut::algod::v2::message::NodeStatus> {
        self.with_rate_limit(|| async {
            self.algod.status().await.map_err(|e| anyhow::anyhow!("{}", e))
        }).await
    }

    /// Get account information
    pub async fn account_information(&self, address: &Address) -> Result<algonaut::algod::v2::message::Account> {
        let addr = *address;
        self.with_rate_limit(|| async {
            self.algod.account_information(&addr).await.map_err(|e| anyhow::anyhow!("{}", e))
        }).await
    }

    /// Get application box
    pub async fn application_box(&self, box_name: &[u8]) -> Result<algonaut::algod::v2::message::Box> {
        let name = box_name.to_vec();
        let app_id = self.app_id;
        self.with_rate_limit(|| async {
            self.algod.application_box(app_id, &name).await.map_err(|e| anyhow::anyhow!("{}", e))
        }).await
    }

    /// Get suggested transaction params
    pub async fn suggested_transaction_params(&self) -> Result<algonaut::algod::v2::message::TransactionParams> {
        self.with_rate_limit(|| async {
            self.algod.suggested_transaction_params().await.map_err(|e| anyhow::anyhow!("{}", e))
        }).await
    }

    /// Broadcast signed transaction
    pub async fn broadcast_signed_transaction(&self, signed_txn: &algonaut::transaction::SignedTransaction) -> Result<algonaut::algod::v2::message::PendingTransaction> {
        // Clone the signed transaction for the closure
        let txn = signed_txn.clone();
        self.with_rate_limit(|| async {
            self.algod.broadcast_signed_transaction(&txn).await.map_err(|e| anyhow::anyhow!("{}", e))
        }).await
    }

    /// Get pending transaction info
    pub async fn pending_transaction_with_id(&self, tx_id: &str) -> Result<algonaut::algod::v2::message::PendingTransaction> {
        let id = tx_id.to_string();
        self.with_rate_limit(|| async {
            self.algod.pending_transaction_with_id(&id).await.map_err(|e| anyhow::anyhow!("{}", e))
        }).await
    }

    /// Get rate limiter stats
    pub fn rate_limit_stats(&self) -> (u64, u64) {
        self.rate_limiter.stats()
    }
}

// =============================================================================
// Configuration
// =============================================================================

/// Algorand node configuration
struct AlgoConfig {
    algod_url: String,
    algod_token: String,
    app_id: u64,
    network: String,
}

impl AlgoConfig {
    fn from_env() -> Self {
        let network = std::env::var("ALGO_NETWORK").unwrap_or_else(|_| "mainnet".into());

        // Default to Nodely endpoints based on network
        let default_url = match network.as_str() {
            "testnet" => NODELY_TESTNET_ALGOD,
            _ => NODELY_MAINNET_ALGOD,
        };

        Self {
            algod_url: std::env::var("ALGOD_URL").unwrap_or_else(|_| default_url.into()),
            // Nodely free tier doesn't require a token
            algod_token: std::env::var("ALGOD_TOKEN").unwrap_or_else(|_| "".into()),
            app_id: std::env::var("APP_ID").unwrap_or_else(|_| "0".into()).parse().unwrap_or(0),
            network,
        }
    }
}

// =============================================================================
// Main
// =============================================================================

#[tokio::main]
async fn main() -> Result<()> {
    let config = AlgoConfig::from_env();

    println!("fPOW CLI - Algorand Client");
    println!("Network: {}", config.network);
    println!("Endpoint: {}", config.algod_url);
    println!();

    // Read account from mnemonic
    let mnemonic = std::env::var("MNEMONIC").expect("Missing MNEMONIC env var");
    let account = Account::from_mnemonic(&mnemonic)?;

    // Create rate-limited Algod client
    let client = AlgoClient::new(&config.algod_url, &config.algod_token, config.app_id)?;

    // Process command
    let command = std::env::var("COMMAND").unwrap_or_else(|_| "help".into());
    match command.as_str() {
        "status" => {
            status(&client).await?;
        }
        "account" => {
            log_account(&client, &account).await?;
        }
        "board" => {
            log_board(&client).await?;
        }
        "treasury" => {
            log_treasury(&client).await?;
        }
        "miner" => {
            log_miner(&client, &account).await?;
        }
        "claim_algo" => {
            claim_algo(&client, &account).await?;
        }
        "claim_fpow" => {
            claim_fpow(&client, &account).await?;
        }
        "deploy" => {
            deploy(&client, &account).await?;
        }
        "deposit" => {
            deposit(&client, &account).await?;
        }
        "withdraw" => {
            withdraw(&client, &account).await?;
        }
        "claim_yield" => {
            claim_yield(&client, &account).await?;
        }
        "stats" => {
            let (daily, backoff) = client.rate_limit_stats();
            println!("Rate Limit Stats");
            println!("  Daily requests: {}/{}", daily, MAX_DAILY_REQUESTS);
            println!("  Current backoff: {}ms", backoff);
        }
        "help" | _ => {
            println!("Available commands:");
            println!("  status       - Show Algorand node status");
            println!("  account      - Show account info");
            println!("  board        - Show board state");
            println!("  treasury     - Show treasury state");
            println!("  miner        - Show miner state");
            println!("  claim_algo   - Claim ALGO rewards");
            println!("  claim_fpow   - Claim fPOW rewards");
            println!("  deploy       - Deploy ALGO to squares");
            println!("  deposit      - Deposit fPOW stake");
            println!("  withdraw     - Withdraw fPOW stake");
            println!("  claim_yield  - Claim staking yield");
            println!("  stats        - Show rate limit stats");
            println!();
            println!("Environment variables:");
            println!("  MNEMONIC      - Wallet mnemonic (required)");
            println!("  APP_ID        - fPOW application ID");
            println!("  ALGO_NETWORK  - Network: mainnet (default) or testnet");
            println!("  ALGOD_URL     - Custom Algod URL (default: Nodely)");
            println!("  ALGOD_TOKEN   - Algod API token (optional for Nodely)");
            println!();
            println!("Rate Limiting:");
            println!("  - Max {} requests/second", MAX_REQUESTS_PER_SECOND);
            println!("  - Max {} requests/day", MAX_DAILY_REQUESTS);
            println!("  - Automatic exponential backoff on 429 errors");
        }
    }

    // Print final rate limit stats
    let (daily, backoff) = client.rate_limit_stats();
    if daily > 0 {
        println!();
        println!("Session stats: {} requests, {}ms backoff", daily, backoff);
    }

    Ok(())
}

// =============================================================================
// Commands
// =============================================================================

/// Show Algorand node status
async fn status(client: &AlgoClient) -> Result<()> {
    let status = client.status().await?;
    println!("Algorand Node Status");
    println!("  Last round: {}", status.last_round);
    println!("  Time since last round: {} ms", status.time_since_last_round.as_millis());
    println!("  Catchup time: {} ms", status.catchup_time.as_millis());
    Ok(())
}

/// Show account info
async fn log_account(client: &AlgoClient, account: &Account) -> Result<()> {
    let address = account.address();
    let info = client.account_information(&address).await?;
    println!("Account Info");
    println!("  Address: {}", address);
    println!("  Balance: {} ALGO", MicroAlgos(info.amount).to_algos());
    println!("  Min balance: {} ALGO", MicroAlgos(info.min_balance).to_algos());
    println!("  Pending rewards: {} ALGO", MicroAlgos(info.pending_rewards).to_algos());
    Ok(())
}

/// Show board state from box storage
async fn log_board(client: &AlgoClient) -> Result<()> {
    let box_name = board_box_name();
    match client.application_box(&box_name).await {
        Ok(box_data) => {
            println!("Board State");
            println!("  Box name: {:?}", String::from_utf8_lossy(&box_name));
            println!("  Data length: {} bytes", box_data.value.len());
            if box_data.value.len() >= 32 {
                let round_id = u64::from_le_bytes(box_data.value[0..8].try_into()?);
                let start_round = u64::from_le_bytes(box_data.value[8..16].try_into()?);
                let end_round = u64::from_le_bytes(box_data.value[16..24].try_into()?);
                let epoch_id = u64::from_le_bytes(box_data.value[24..32].try_into()?);
                println!("  Round ID: {}", round_id);
                println!("  Start round: {}", start_round);
                println!("  End round: {}", end_round);
                println!("  Epoch ID: {}", epoch_id);
            }
        }
        Err(e) => {
            println!("Failed to get board box: {}", e);
        }
    }
    Ok(())
}

/// Show treasury state from box storage
async fn log_treasury(client: &AlgoClient) -> Result<()> {
    let box_name = treasury_box_name();
    match client.application_box(&box_name).await {
        Ok(box_data) => {
            println!("Treasury State");
            println!("  Box name: {:?}", String::from_utf8_lossy(&box_name));
            println!("  Data length: {} bytes", box_data.value.len());
            if box_data.value.len() >= 8 {
                let balance = u64::from_le_bytes(box_data.value[0..8].try_into()?);
                println!("  Balance: {} microALGO", balance);
            }
        }
        Err(e) => {
            println!("Failed to get treasury box: {}", e);
        }
    }
    Ok(())
}

/// Show miner state from box storage
async fn log_miner(client: &AlgoClient, account: &Account) -> Result<()> {
    let authority = std::env::var("AUTHORITY")
        .map(|s| Address::from_str(&s).expect("Invalid AUTHORITY"))
        .unwrap_or_else(|_| account.address());

    let box_name = miner_box_name(&authority.0);
    match client.application_box(&box_name).await {
        Ok(box_data) => {
            println!("Miner State");
            println!("  Authority: {}", authority);
            println!("  Box name length: {} bytes", box_name.len());
            println!("  Data length: {} bytes", box_data.value.len());
        }
        Err(e) => {
            println!("Failed to get miner box: {}", e);
            println!("Miner account may not exist yet.");
        }
    }
    Ok(())
}

/// Build and submit an application call transaction
async fn submit_app_call(
    client: &AlgoClient,
    account: &Account,
    method: FpowInstruction,
    args: Vec<Vec<u8>>,
    boxes: Vec<Vec<u8>>,
) -> Result<String> {
    // Get suggested parameters
    let params = client.suggested_transaction_params().await?;

    // Build method selector
    let method_selector = {
        use sha2::{Sha512_256, Digest};
        let selector = method.method_selector();
        let hash = Sha512_256::digest(selector.as_bytes());
        hash[0..4].to_vec()
    };

    // Build application arguments
    let mut app_args = vec![method_selector];
    app_args.extend(args);

    // Build box references
    let app_id = std::env::var("APP_ID").unwrap_or_else(|_| "0".into()).parse().unwrap_or(0);
    let box_refs: Vec<(u64, Vec<u8>)> = boxes.into_iter().map(|b| (app_id, b)).collect();

    // Build transaction
    let txn = TxnBuilder::with(
        &params,
        CallApplication::new(account.address(), app_id)
            .app_arguments(app_args)
            .boxes(box_refs)
            .build(),
    )
    .build()?;

    // Sign transaction
    let signed_txn = account.sign_transaction(txn)?;

    // Submit transaction (rate limited)
    let pending = client.broadcast_signed_transaction(&signed_txn).await?;
    println!("Transaction submitted: {}", pending.tx_id);

    // Wait for confirmation (with polling delay for rate limiting)
    loop {
        sleep(Duration::from_millis(500)).await; // Rate-limited polling
        match client.pending_transaction_with_id(&pending.tx_id).await {
            Ok(confirmed) => {
                if confirmed.confirmed_round.is_some() {
                    println!("Transaction confirmed in round: {:?}", confirmed.confirmed_round);
                    return Ok(pending.tx_id.to_string());
                }
            }
            Err(e) => {
                // Transaction may still be pending
                if !e.to_string().contains("pending") {
                    return Err(e);
                }
            }
        }
    }
}

/// Claim ALGO rewards
async fn claim_algo(client: &AlgoClient, account: &Account) -> Result<()> {
    let miner_box = miner_box_name(&account.address().0);

    println!("Claiming ALGO rewards...");
    let tx_id = submit_app_call(
        client,
        account,
        FpowInstruction::ClaimALGO,
        vec![],
        vec![miner_box],
    )
    .await?;
    println!("Claim ALGO transaction: {}", tx_id);

    Ok(())
}

/// Claim fPOW rewards
async fn claim_fpow(client: &AlgoClient, account: &Account) -> Result<()> {
    let miner_box = miner_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Claiming fPOW rewards...");
    let tx_id = submit_app_call(
        client,
        account,
        FpowInstruction::ClaimFPOW,
        vec![],
        vec![miner_box, treasury_box],
    )
    .await?;
    println!("Claim fPOW transaction: {}", tx_id);

    Ok(())
}

/// Deploy ALGO to squares
async fn deploy(client: &AlgoClient, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let square: u64 = std::env::var("SQUARE")
        .expect("Missing SQUARE env var")
        .parse()
        .expect("Invalid SQUARE");

    let mask: u32 = 1 << square;

    let miner_box = miner_box_name(&account.address().0);
    let board_box = board_box_name();

    println!("Deploying {} microALGO to square {}...", amount, square);
    let tx_id = submit_app_call(
        client,
        account,
        FpowInstruction::Deploy,
        vec![
            amount.to_be_bytes().to_vec(),
            mask.to_be_bytes().to_vec(),
        ],
        vec![miner_box, board_box],
    )
    .await?;
    println!("Deploy transaction: {}", tx_id);

    Ok(())
}

/// Deposit fPOW stake
async fn deposit(client: &AlgoClient, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let stake_box = stake_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Depositing {} fPOW...", amount);
    let tx_id = submit_app_call(
        client,
        account,
        FpowInstruction::Deposit,
        vec![
            amount.to_be_bytes().to_vec(),
            0u64.to_be_bytes().to_vec(),
        ],
        vec![stake_box, treasury_box],
    )
    .await?;
    println!("Deposit transaction: {}", tx_id);

    Ok(())
}

/// Withdraw fPOW stake
async fn withdraw(client: &AlgoClient, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let stake_box = stake_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Withdrawing {} fPOW...", amount);
    let tx_id = submit_app_call(
        client,
        account,
        FpowInstruction::Withdraw,
        vec![amount.to_be_bytes().to_vec()],
        vec![stake_box, treasury_box],
    )
    .await?;
    println!("Withdraw transaction: {}", tx_id);

    Ok(())
}

/// Claim staking yield
async fn claim_yield(client: &AlgoClient, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let stake_box = stake_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Claiming {} fPOW yield...", amount);
    let tx_id = submit_app_call(
        client,
        account,
        FpowInstruction::ClaimYield,
        vec![amount.to_be_bytes().to_vec()],
        vec![stake_box, treasury_box],
    )
    .await?;
    println!("Claim yield transaction: {}", tx_id);

    Ok(())
}
