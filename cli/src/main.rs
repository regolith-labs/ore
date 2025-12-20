//! fPOW CLI - Algorand Client
//!
//! Command-line interface for interacting with the fPOW smart contract on Algorand.

use std::str::FromStr;

use algonaut::algod::v2::Algod;
use algonaut::core::{Address, MicroAlgos};
use algonaut::transaction::{
    account::Account,
    builder::CallApplication,
    Transaction, TxnBuilder,
};
use anyhow::Result;
use fpow_api::prelude::*;

/// Algorand node configuration
struct AlgoConfig {
    algod_url: String,
    algod_token: String,
    app_id: u64,
}

impl AlgoConfig {
    fn from_env() -> Self {
        Self {
            algod_url: std::env::var("ALGOD_URL").unwrap_or_else(|_| "http://localhost:4001".into()),
            algod_token: std::env::var("ALGOD_TOKEN").unwrap_or_else(|_| "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into()),
            app_id: std::env::var("APP_ID").unwrap_or_else(|_| "0".into()).parse().unwrap_or(0),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = AlgoConfig::from_env();

    // Read account from mnemonic
    let mnemonic = std::env::var("MNEMONIC").expect("Missing MNEMONIC env var");
    let account = Account::from_mnemonic(&mnemonic)?;

    // Create Algod client
    let algod = Algod::new(&config.algod_url, &config.algod_token)?;

    // Process command
    let command = std::env::var("COMMAND").expect("Missing COMMAND env var");
    match command.as_str() {
        "status" => {
            status(&algod).await?;
        }
        "account" => {
            log_account(&algod, &account).await?;
        }
        "board" => {
            log_board(&algod, config.app_id).await?;
        }
        "treasury" => {
            log_treasury(&algod, config.app_id).await?;
        }
        "miner" => {
            log_miner(&algod, config.app_id, &account).await?;
        }
        "claim_algo" => {
            claim_algo(&algod, config.app_id, &account).await?;
        }
        "claim_fpow" => {
            claim_fpow(&algod, config.app_id, &account).await?;
        }
        "deploy" => {
            deploy(&algod, config.app_id, &account).await?;
        }
        "deposit" => {
            deposit(&algod, config.app_id, &account).await?;
        }
        "withdraw" => {
            withdraw(&algod, config.app_id, &account).await?;
        }
        "claim_yield" => {
            claim_yield(&algod, config.app_id, &account).await?;
        }
        _ => {
            println!("Unknown command: {}", command);
            println!();
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
        }
    }

    Ok(())
}

/// Show Algorand node status
async fn status(algod: &Algod) -> Result<()> {
    let status = algod.status().await?;
    println!("Algorand Node Status");
    println!("  Last round: {}", status.last_round);
    println!("  Time since last round: {} ms", status.time_since_last_round.as_millis());
    println!("  Catchup time: {} ms", status.catchup_time.as_millis());
    Ok(())
}

/// Show account info
async fn log_account(algod: &Algod, account: &Account) -> Result<()> {
    let address = account.address();
    let info = algod.account_information(&address).await?;
    println!("Account Info");
    println!("  Address: {}", address);
    println!("  Balance: {} ALGO", MicroAlgos(info.amount).to_algos());
    println!("  Min balance: {} ALGO", MicroAlgos(info.min_balance).to_algos());
    println!("  Pending rewards: {} ALGO", MicroAlgos(info.pending_rewards).to_algos());
    Ok(())
}

/// Show board state from box storage
async fn log_board(algod: &Algod, app_id: u64) -> Result<()> {
    let box_name = board_box_name();
    match algod.application_box(app_id, &box_name).await {
        Ok(box_data) => {
            println!("Board State");
            println!("  Box name: {:?}", String::from_utf8_lossy(&box_name));
            println!("  Data length: {} bytes", box_data.value.len());
            // Deserialize board data
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
async fn log_treasury(algod: &Algod, app_id: u64) -> Result<()> {
    let box_name = treasury_box_name();
    match algod.application_box(app_id, &box_name).await {
        Ok(box_data) => {
            println!("Treasury State");
            println!("  Box name: {:?}", String::from_utf8_lossy(&box_name));
            println!("  Data length: {} bytes", box_data.value.len());
            // Deserialize treasury data
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
async fn log_miner(algod: &Algod, app_id: u64, account: &Account) -> Result<()> {
    let authority = std::env::var("AUTHORITY")
        .map(|s| Address::from_str(&s).expect("Invalid AUTHORITY"))
        .unwrap_or_else(|_| account.address());

    let box_name = miner_box_name(&authority.0);
    match algod.application_box(app_id, &box_name).await {
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
    algod: &Algod,
    app_id: u64,
    account: &Account,
    method: FpowInstruction,
    args: Vec<Vec<u8>>,
    boxes: Vec<Vec<u8>>,
) -> Result<String> {
    // Get suggested parameters
    let params = algod.suggested_transaction_params().await?;

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

    // Submit transaction
    let pending = algod.broadcast_signed_transaction(&signed_txn).await?;
    println!("Transaction submitted: {}", pending.tx_id);

    // Wait for confirmation
    let confirmed = algod.pending_transaction_with_id(&pending.tx_id).await?;
    println!("Transaction confirmed in round: {:?}", confirmed.confirmed_round);

    Ok(pending.tx_id.to_string())
}

/// Claim ALGO rewards
async fn claim_algo(algod: &Algod, app_id: u64, account: &Account) -> Result<()> {
    let miner_box = miner_box_name(&account.address().0);

    println!("Claiming ALGO rewards...");
    let tx_id = submit_app_call(
        algod,
        app_id,
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
async fn claim_fpow(algod: &Algod, app_id: u64, account: &Account) -> Result<()> {
    let miner_box = miner_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Claiming fPOW rewards...");
    let tx_id = submit_app_call(
        algod,
        app_id,
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
async fn deploy(algod: &Algod, app_id: u64, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let square: u64 = std::env::var("SQUARE")
        .expect("Missing SQUARE env var")
        .parse()
        .expect("Invalid SQUARE");

    // Build square mask
    let mask: u32 = 1 << square;

    let miner_box = miner_box_name(&account.address().0);
    let board_box = board_box_name();

    println!("Deploying {} microALGO to square {}...", amount, square);
    let tx_id = submit_app_call(
        algod,
        app_id,
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
async fn deposit(algod: &Algod, app_id: u64, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let stake_box = stake_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Depositing {} fPOW...", amount);
    let tx_id = submit_app_call(
        algod,
        app_id,
        account,
        FpowInstruction::Deposit,
        vec![
            amount.to_be_bytes().to_vec(),
            0u64.to_be_bytes().to_vec(), // compound_fee
        ],
        vec![stake_box, treasury_box],
    )
    .await?;
    println!("Deposit transaction: {}", tx_id);

    Ok(())
}

/// Withdraw fPOW stake
async fn withdraw(algod: &Algod, app_id: u64, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let stake_box = stake_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Withdrawing {} fPOW...", amount);
    let tx_id = submit_app_call(
        algod,
        app_id,
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
async fn claim_yield(algod: &Algod, app_id: u64, account: &Account) -> Result<()> {
    let amount: u64 = std::env::var("AMOUNT")
        .expect("Missing AMOUNT env var")
        .parse()
        .expect("Invalid AMOUNT");

    let stake_box = stake_box_name(&account.address().0);
    let treasury_box = treasury_box_name();

    println!("Claiming {} fPOW yield...", amount);
    let tx_id = submit_app_call(
        algod,
        app_id,
        account,
        FpowInstruction::ClaimYield,
        vec![amount.to_be_bytes().to_vec()],
        vec![stake_box, treasury_box],
    )
    .await?;
    println!("Claim yield transaction: {}", tx_id);

    Ok(())
}
