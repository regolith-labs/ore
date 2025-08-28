use std::str::FromStr;

use ore_api::prelude::*;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    client_error::{reqwest::StatusCode, ClientErrorKind},
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    keccak::hash,
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use spl_token::amount_to_ui_amount;
use steel::{AccountDeserialize, Clock, Discriminator};

#[tokio::main]
async fn main() {
    // Read keypair from file
    let payer =
        read_keypair_file(&std::env::var("KEYPAIR").expect("Missing KEYPAIR env var")).unwrap();

    // Build transaction
    let rpc = RpcClient::new(std::env::var("RPC").expect("Missing RPC env var"));
    match std::env::var("COMMAND")
        .expect("Missing COMMAND env var")
        .as_str()
    {
        "clock" => {
            log_clock(&rpc).await.unwrap();
        }
        "claim" => {
            claim(&rpc, &payer).await.unwrap();
        }
        "close" => {
            close(&rpc, &payer).await.unwrap();
        }
        "close_all" => {
            close_all(&rpc, &payer).await.unwrap();
        }
        "market" => {
            log_market(&rpc).await.unwrap();
        }
        "block" => {
            log_block(&rpc).await.unwrap();
        }
        "blocks" => {
            log_blocks(&rpc).await.unwrap();
        }
        "config" => {
            log_config(&rpc).await.unwrap();
        }
        "mine" => {
            mine(&rpc, &payer).await.unwrap();
        }
        "initialize" => {
            initialize(&rpc, &payer).await.unwrap();
        }
        "open" => {
            open(&rpc, &payer).await.unwrap();
        }
        "swap" => {
            swap(&rpc, &payer).await.unwrap();
        }
        "reset" => {
            reset(&rpc, &payer).await.unwrap();
        }
        "treasury" => {
            log_treasury(&rpc).await.unwrap();
        }
        "miner" => {
            log_miner(&rpc, &payer).await.unwrap();
        }
        "set_admin" => {
            set_admin(&rpc, &payer).await.unwrap();
        }
        "set_fee_collector" => {
            set_fee_collector(&rpc, &payer).await.unwrap();
        }
        "benchmark" => {
            benchmark_keccak().await.unwrap();
        }
        _ => panic!("Invalid command"),
    };
}

async fn initialize(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix = ore_api::sdk::initialize(payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn close(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let block = get_block(rpc, id).await?;
    let ix = ore_api::sdk::close(payer.pubkey(), block.opener, block.best_hash_miner, id);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn claim(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix = ore_api::sdk::claim(payer.pubkey(), u64::MAX);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn close_all(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let clock = get_clock(rpc).await?;
    let blocks = get_blocks(rpc).await?;
    println!("Closing all blocks... {}", blocks.len());
    for (_, block) in blocks {
        println!("Closing block {}", block.id);
        if clock.slot > block.end_slot + MINING_WINDOW {
            let ix = ore_api::sdk::close(
                payer.pubkey(),
                block.opener,
                block.best_hash_miner,
                block.id,
            );
            submit_transaction(rpc, payer, &[ix]).await?;
        }
    }
    Ok(())
}

async fn mine(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let miner = get_miner(rpc, payer.pubkey()).await?;
    let block = get_block(rpc, miner.block_id).await?;
    let clock = get_clock(rpc).await?;
    if clock.slot < block.end_slot {
        return Err(anyhow::anyhow!("Mining window is not yet open."));
    }
    if clock.slot >= block.end_slot + MINING_WINDOW {
        return Err(anyhow::anyhow!("Mining window is closed."));
    }
    let mut best_hash = [u8::MAX; 32];
    let mut best_nonce = 0;
    for i in 0..miner.hashpower {
        let mut seed = [0u8; 112];
        seed[..8].copy_from_slice(&block.id.to_le_bytes());
        seed[8..40].copy_from_slice(&block.slot_hash);
        seed[40..72].copy_from_slice(&miner.authority.to_bytes());
        seed[72..104].copy_from_slice(&miner.seed);
        seed[104..].copy_from_slice(&i.to_le_bytes());
        let h = hash(&seed).to_bytes();
        if h < best_hash {
            best_hash = h;
            best_nonce = i;
        }
    }
    if block.best_hash < best_hash {
        return Err(anyhow::anyhow!("A better hash was already found."));
    }
    println!("Found best hash: {:?}", best_hash.to_ascii_lowercase());
    let ix = ore_api::sdk::mine(payer.pubkey(), block.id, best_nonce);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn reset(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let market = get_market(rpc).await?;
    let id = market.block_id;
    let open_ix = ore_api::sdk::open(payer.pubkey(), id + 1);
    let reset_ix = ore_api::sdk::reset(payer.pubkey(), id);
    submit_transaction(rpc, payer, &[open_ix, reset_ix]).await?;
    Ok(())
}

async fn open(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let ix = ore_api::sdk::open(payer.pubkey(), id);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn swap(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let market = get_market(rpc).await?;
    let id = market.block_id;
    let config = get_config(rpc).await?;
    let fee_collector = config.fee_collector;
    let ix = ore_api::sdk::swap(
        payer.pubkey(),
        id,
        fee_collector,
        1_000_000_000,
        SwapDirection::Buy,
        SwapPrecision::ExactIn,
        [0; 32],
    );
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn set_admin(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix = ore_api::sdk::set_admin(payer.pubkey(), payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn set_fee_collector(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let fee_collector = std::env::var("FEE_COLLECTOR").expect("Missing FEE_COLLECTOR env var");
    let fee_collector = Pubkey::from_str(&fee_collector).expect("Invalid FEE_COLLECTOR");
    let ix = ore_api::sdk::set_fee_collector(payer.pubkey(), fee_collector);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn log_treasury(_rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let treasury_address = ore_api::state::treasury_pda().0;
    println!("Treasury");
    println!("  address: {}", treasury_address);
    Ok(())
}

async fn log_miner(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let authority = std::env::var("AUTHORITY").unwrap_or(payer.pubkey().to_string());
    let authority = Pubkey::from_str(&authority).expect("Invalid AUTHORITY");
    let miner_address = ore_api::state::miner_pda(authority).0;
    let miner = get_miner(&rpc, authority).await?;
    println!("Miner");
    println!("  address: {}", miner_address);
    println!("  authority: {}", authority);
    println!("  block_id: {}", miner.block_id);
    println!("  hashpower: {}", miner.hashpower);
    println!("  total_hashpower: {}", miner.total_hashpower);
    println!("  total_rewards: {}", miner.total_rewards);
    Ok(())
}

async fn log_clock(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let clock = get_clock(&rpc).await?;
    println!("Clock");
    println!("  slot: {}", clock.slot);
    println!("  epoch_start_timestamp: {}", clock.epoch_start_timestamp);
    println!("  epoch: {}", clock.epoch);
    println!("  leader_schedule_epoch: {}", clock.leader_schedule_epoch);
    println!("  unix_timestamp: {}", clock.unix_timestamp);
    Ok(())
}

async fn log_config(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let config = get_config(&rpc).await?;
    println!("Config");
    println!("  admin: {}", config.admin);
    println!("  block_duration: {}", config.block_duration);
    println!("  sniper_fee_duration: {}", config.sniper_fee_duration);
    println!("  fee_collector: {}", config.fee_collector);
    println!("  fee_rate: {}", config.fee_rate);
    Ok(())
}

async fn log_market(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let market = get_market(&rpc).await?;
    let block = get_block(&rpc, market.block_id).await?;
    let clock = get_clock(rpc).await?;
    print_market(market);
    println!("");
    print_block(block, &clock);
    Ok(())
}

async fn log_block(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let block = get_block(&rpc, id).await?;
    let clock = get_clock(&rpc).await?;
    print_block(block, &clock);
    Ok(())
}

fn print_block(block: Block, clock: &Clock) {
    let current_slot = clock.slot;
    println!("Block");
    println!("  Id: {:?}", block.id);
    println!("  Slot hash: {:?}", block.slot_hash);
    println!("  Total hashpower: {}", block.total_hashpower);
    println!("  Best hash: {:?}", block.best_hash);
    println!("  Best hash miner: {:?}", block.best_hash_miner);
    println!("  Start slot: {}", block.start_slot);
    println!("  End slot: {}", block.end_slot);
    println!("  Reward: {}", block.reward);
    println!(
        "  Time remaining: {} sec",
        (block.end_slot.saturating_sub(current_slot) as f64) * 0.4
    );
}

fn print_market(market: Market) {
    println!("Market");
    println!("  Block id: {}", market.block_id);
    println!("  Base token: {:?}", market.base);
    println!("  Quote token: {:?}", market.quote);
    println!("  Fee: {:?}", market.fee);
    println!("  Snapshot: {:?}", market.snapshot);
    let price = amount_to_ui_amount(market.quote.liquidity() as u64, TOKEN_DECIMALS)
        / market.base.liquidity() as f64;
    println!("  Price: {:.11?} ORE / hash", price);
}

async fn log_blocks(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let clock = get_clock(&rpc).await?;
    let mut blocks = get_blocks(&rpc).await?;
    blocks.sort_by_key(|(_, block)| block.id);
    for (_, block) in blocks {
        print_block(block, &clock);
    }
    Ok(())
}

async fn get_block(rpc: &RpcClient, id: u64) -> Result<Block, anyhow::Error> {
    let block_pda = ore_api::state::block_pda(id);
    let account = rpc.get_account(&block_pda.0).await?;
    let block = Block::try_from_bytes(&account.data)?;
    Ok(*block)
}

async fn get_config(rpc: &RpcClient) -> Result<Config, anyhow::Error> {
    let config_pda = ore_api::state::config_pda();
    let account = rpc.get_account(&config_pda.0).await?;
    let config = Config::try_from_bytes(&account.data)?;
    Ok(*config)
}

async fn get_market(rpc: &RpcClient) -> Result<Market, anyhow::Error> {
    let market_pda = ore_api::state::market_pda();
    let account = rpc.get_account(&market_pda.0).await?;
    let market = Market::try_from_bytes(&account.data)?;
    Ok(*market)
}

async fn get_miner(rpc: &RpcClient, authority: Pubkey) -> Result<Miner, anyhow::Error> {
    let miner_pda = ore_api::state::miner_pda(authority);
    let account = rpc.get_account(&miner_pda.0).await?;
    let miner = Miner::try_from_bytes(&account.data)?;
    Ok(*miner)
}

async fn get_clock(rpc: &RpcClient) -> Result<Clock, anyhow::Error> {
    let data = rpc.get_account_data(&solana_sdk::sysvar::clock::ID).await?;
    let clock = bincode::deserialize::<Clock>(&data)?;
    Ok(clock)
}

async fn get_blocks(rpc: &RpcClient) -> Result<Vec<(Pubkey, Block)>, anyhow::Error> {
    let blocks = get_program_accounts::<Block>(rpc, ore_api::ID, vec![]).await?;
    Ok(blocks)
}

async fn _simulate_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) {
    let blockhash = rpc.get_latest_blockhash().await.unwrap();
    let x = rpc
        .simulate_transaction(&Transaction::new_signed_with_payer(
            instructions,
            Some(&payer.pubkey()),
            &[payer],
            blockhash,
        ))
        .await;
    println!("Simulation result: {:?}", x);
}

async fn submit_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    let blockhash = rpc.get_latest_blockhash().await?;
    let mut all_instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(1_400_000),
        ComputeBudgetInstruction::set_compute_unit_price(1_000_000),
    ];
    all_instructions.extend_from_slice(instructions);
    let transaction = Transaction::new_signed_with_payer(
        &all_instructions,
        Some(&payer.pubkey()),
        &[payer],
        blockhash,
    );

    match rpc.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => {
            println!("Transaction submitted: {:?}", signature);
            Ok(signature)
        }
        Err(e) => {
            println!("Error submitting transaction: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn get_program_accounts<T>(
    client: &RpcClient,
    program_id: Pubkey,
    filters: Vec<RpcFilterType>,
) -> Result<Vec<(Pubkey, T)>, anyhow::Error>
where
    T: AccountDeserialize + Discriminator + Clone,
{
    let mut all_filters = vec![RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        0,
        &T::discriminator().to_le_bytes(),
    ))];
    all_filters.extend(filters);
    let result = client
        .get_program_accounts_with_config(
            &program_id,
            RpcProgramAccountsConfig {
                filters: Some(all_filters),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .await;

    match result {
        Ok(accounts) => {
            let accounts = accounts
                .into_iter()
                .filter_map(|(pubkey, account)| {
                    if let Ok(account) = T::try_from_bytes(&account.data) {
                        Some((pubkey, account.clone()))
                    } else {
                        None
                    }
                })
                .collect();
            Ok(accounts)
        }
        Err(err) => match err.kind {
            ClientErrorKind::Reqwest(err) => {
                if let Some(status_code) = err.status() {
                    if status_code == StatusCode::GONE {
                        panic!(
                                "\n{} Your RPC provider does not support the getProgramAccounts endpoint, needed to execute this command. Please use a different RPC provider.\n",
                                "ERROR"
                            );
                    }
                }
                return Err(anyhow::anyhow!("Failed to get program accounts: {}", err));
            }
            _ => return Err(anyhow::anyhow!("Failed to get program accounts: {}", err)),
        },
    }
}

async fn benchmark_keccak() -> Result<(), anyhow::Error> {
    use solana_program::keccak::hash;
    use std::time::Instant;

    const NUM_HASHES: u64 = 1_000_000;
    let start = Instant::now();

    for i in 0..NUM_HASHES {
        let _ = hash(&i.to_le_bytes());
    }

    let duration = start.elapsed();
    let hashes_per_sec = NUM_HASHES as f64 / duration.as_secs_f64();

    println!("\nKeccak-256 Benchmark:");
    println!("Time elapsed: {:.2?}", duration);
    println!("Hashes computed: {}", NUM_HASHES);
    println!("Hashes per second: {:.0}", hashes_per_sec);

    Ok(())
}
