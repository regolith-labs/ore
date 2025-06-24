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
        "open" => {
            open(&rpc, &payer).await.unwrap();
        }
        "close" => {
            close(&rpc, &payer).await.unwrap();
        }
        "clock" => {
            log_clock(&rpc).await.unwrap();
        }
        "block" => {
            log_block(&rpc).await.unwrap();
        }
        "blocks" => {
            log_blocks(&rpc).await.unwrap();
        }
        "deposit" => {
            deposit(&rpc, &payer).await.unwrap();
        }
        "swap" => {
            swap(&rpc, &payer).await.unwrap();
        }
        "commit" => {
            commit(&rpc, &payer).await.unwrap();
        }
        // "uncommit" => {
        //     uncommit(&rpc, &payer).await.unwrap();
        // }
        _ => panic!("Invalid command"),
    };
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

async fn close(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let ix = ore_api::sdk::close(payer.pubkey(), payer.pubkey(), id);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn commit(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let ix = ore_api::sdk::commit(payer.pubkey(), 10000000, Pubkey::default(), 0, id, [0; 32]);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn deposit(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let ix = ore_api::sdk::deposit(payer.pubkey(), id, 10000000);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn swap(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let ix = ore_api::sdk::swap(
        payer.pubkey(),
        id,
        10000000,
        SwapDirection::Buy,
        SwapPrecision::ExactIn,
    );
    submit_transaction(rpc, payer, &[ix]).await?;
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

async fn log_block(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let block = get_block(&rpc, id).await?;
    let clock = get_clock(&rpc).await?;
    print_block(block, &clock);
    Ok(())
}

fn print_block(block: Block, clock: &Clock) {
    let address = block_pda(block.id).0;
    let current_slot = clock.slot;
    let elapsed_time = (block.start_slot - current_slot) as f64 * 0.4;
    println!("Address: {:?}", address);
    println!("  Id: {:?}", block.id);
    println!("  Start slot: {:?}", block.start_slot);
    println!("  Starts in: {:?} sec", elapsed_time as u64);
    println!("  Slot hash: {:?}", block.slot_hash);
    println!("  Total hashes: {:?}", block.total_committed);
    println!("  Total deployed: {:?}", block.total_deployed);
    println!("  Total rewards: {:?}", block.total_rewards);
    println!("  Lode reward: {:?}", block.reward.lode_reward);
    println!("  Lode authority: {:?}", block.reward.lode_authority);
    println!("  Lode hash: {:?}", block.reward.lode_hash);
    println!("  Nugget reward: {:?}", block.reward.nugget_reward);
    println!("  Nugget threshold: {:?}", block.reward.nugget_threshold);
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

async fn get_clock(rpc: &RpcClient) -> Result<Clock, anyhow::Error> {
    let data = rpc.get_account_data(&solana_sdk::sysvar::clock::ID).await?;
    let clock = bincode::deserialize::<Clock>(&data)?;
    Ok(clock)
}

async fn get_blocks(rpc: &RpcClient) -> Result<Vec<(Pubkey, Block)>, anyhow::Error> {
    let blocks = get_program_accounts::<Block>(rpc, ore_api::ID, vec![]).await?;
    Ok(blocks)
}

async fn submit_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> Result<solana_sdk::signature::Signature, anyhow::Error> {
    let blockhash = rpc.get_latest_blockhash().await?;
    let mut all_instructions = vec![ComputeBudgetInstruction::set_compute_unit_limit(1_400_000)];
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
