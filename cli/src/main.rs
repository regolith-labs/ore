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

async fn log_clock(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let clock = get_clock(&rpc).await?;
    println!("Clock: {:?}", clock);
    Ok(())
}

async fn log_block(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let id_str = std::env::var("ID").expect("Missing ID env var");
    let id = id_str.parse::<u64>()?;
    let block = get_block(&rpc, id).await?;
    println!("Block: {:?}", block);
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

// async fn get_block_commits(rpc: &RpcClient) -> Result<Vec<(Pubkey, Commit)>, anyhow::Error> {
//     let block = get_block(rpc).await?;
//     let filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
//         56,
//         &block.current_round.to_le_bytes(),
//     ));
//     let commits = get_program_accounts::<Commit>(rpc, ore_api::ID, vec![filter]).await?;
//     Ok(commits)
// }

// async fn get_my_commits(
//     rpc: &RpcClient,
//     payer: &solana_sdk::signer::keypair::Keypair,
// ) -> Result<Vec<(Pubkey, Commit)>, anyhow::Error> {
//     let filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
//         16,
//         &payer.pubkey().to_bytes().as_ref(),
//     ));
//     let commits = get_program_accounts::<Commit>(rpc, ore_api::ID, vec![filter]).await?;
//     Ok(commits)
// }

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
                .map(|(pubkey, account)| {
                    let account = T::try_from_bytes(&account.data).unwrap().clone();
                    (pubkey, account)
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
