use meteora_pools_sdk::instructions::Swap;
use ore_api::{prelude::*, sdk::*};
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    client_error::{reqwest::StatusCode, ClientErrorKind},
    nonblocking::rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, RpcFilterType},
};
use solana_program::pubkey;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction,
    keccak::hashv,
    native_token::{lamports_to_sol, sol_to_lamports},
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    transaction::Transaction,
};
use steel::{AccountDeserialize, Clock, Discriminator, Instruction};

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
        "initialize" => {
            let ix = initialize(payer.pubkey());
            submit_transaction(&rpc, &payer, &[ix]).await.unwrap();
        }
        "payout" => {
            let ix = payout(payer.pubkey(), Pubkey::new_unique(), Pubkey::new_unique());
            submit_transaction(&rpc, &payer, &[ix]).await.unwrap();
        }
        "reset" => {
            let ix = reset(payer.pubkey(), ore_boost_api::state::config_pda().0);
            submit_transaction(&rpc, &payer, &[ix]).await.unwrap();
        }
        "block" => {
            let block = get_block(&rpc).await.unwrap();
            println!("Block: {:?}", block);
        }
        "crank" => {
            crank(&rpc, &payer).await.unwrap();
        }
        "bet" => {
            bet(&rpc, &payer, sol_to_lamports(1.0)).await.unwrap();
        }
        "close" => {
            close_all_wagers(&rpc, &payer).await.unwrap();
        }
        "wagers" => {
            let wagers = get_block_wagers(&rpc).await.unwrap();
            println!("Wagers: {:?}", wagers);
        }
        "bury" => {
            bury_ore_sol(&rpc, &payer).await.unwrap();
        }
        _ => panic!("Invalid command"),
    };
}

async fn crank(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), solana_client::client_error::ClientError> {
    loop {
        let block = get_block(rpc).await?;
        let clock = get_clock(rpc).await?;

        if clock.slot >= block.ends_at {
            // Time to payout and reset
            let payout_ix = build_payout_ix(rpc, payer).await?;
            let reset_ix = reset(payer.pubkey(), ore_boost_api::state::config_pda().0);
            submit_transaction(rpc, &payer, &[payout_ix, reset_ix]).await?;
            println!("Submitted payout and reset transaction");
        } else {
            // Calculate and print time remaining
            let slots_remaining = block.ends_at.saturating_sub(clock.slot);
            let seconds_remaining = (slots_remaining as f64) * 0.4;
            println!(
                "Time until payout: {:.1} seconds ({} slots) – {} wagers – {} SOL",
                seconds_remaining,
                slots_remaining,
                block.total_wagers,
                lamports_to_sol(block.cumulative_sum)
            );
        }

        // Wait 3 seconds before next check
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn bet(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    amount: u64,
) -> Result<(), solana_client::client_error::ClientError> {
    // Get current block to get round number
    let block = get_block(rpc).await?;

    // Create WSOL ATA if it doesn't exist
    let wsol_ata = spl_associated_token_account::get_associated_token_address(
        &payer.pubkey(),
        &spl_token::native_mint::ID,
    );
    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &payer.pubkey(),
        &spl_token::native_mint::ID,
        &spl_token::ID,
    );
    let mut ixs = match rpc.get_account(&wsol_ata).await {
        Ok(_) => vec![],
        Err(_) => vec![create_ata_ix],
    };

    // Wrap SOL
    let wrap_ix = solana_sdk::system_instruction::transfer(
        &payer.pubkey(),
        &spl_associated_token_account::get_associated_token_address(
            &payer.pubkey(),
            &spl_token::native_mint::ID,
        ),
        amount,
    );
    let sync_native_ix = spl_token::instruction::sync_native(
        &spl_token::ID,
        &spl_associated_token_account::get_associated_token_address(
            &payer.pubkey(),
            &spl_token::native_mint::ID,
        ),
    )
    .unwrap();

    // Build bet instruction
    let seed = generate_seed(&payer, &block);
    println!("Seed: {:?}", seed);
    let ix = ore_api::sdk::bet(
        payer.pubkey(),
        spl_token::native_mint::ID,
        amount,
        block.current_round,
        seed,
    );
    ixs.push(wrap_ix);
    ixs.push(sync_native_ix);
    ixs.push(ix);

    // Submit transaction
    submit_transaction(rpc, payer, &ixs).await?;
    println!("Placed bet of {} lamports", amount);

    Ok(())
}

async fn bury_ore_sol(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), solana_client::client_error::ClientError> {
    let swap = Swap {
        pool: pubkey!("GgaDTFbqdgjoZz3FP7zrtofGwnRS4E6MCzmmD5Ni1Mxj"),
        user_source_token: spl_token::native_mint::ID,
        user_destination_token: ore_api::consts::MINT_ADDRESS,
        a_vault: pubkey!("3s6ki6dQSM8FuqWiPsnGkgVsAEo8BTAfUR1Vvt1TPiJN"),
        b_vault: pubkey!("FERjPVNEa7Udq8CEv68h6tPL46Tq7ieE49HrE2wea3XT"),
        a_token_vault: pubkey!("BtJuiRG44vew5nYBVeUhuBawPTZLyYYxdzTYzerkfnto"),
        b_token_vault: pubkey!("HZeLxbZ9uHtSpwZC3LBr4Nubd14iHwz7bRSghRZf5VCG"),
        a_vault_lp_mint: pubkey!("6Av9sdKvnjwoDHVnhEiz6JEq8e6SGzmhCsCncT2WJ7nN"),
        b_vault_lp_mint: pubkey!("FZN7QZ8ZUUAxMPfxYEYkH3cXUASzH8EqA6B4tyCL8f1j"),
        a_vault_lp: pubkey!("2k7V1NtM1krwh1sdt5wWqBRcvNQ5jzxj3J2rV78zdTsL"),
        b_vault_lp: pubkey!("CFATQFgkKXJyU3MdCNvQqN79qorNSMJFF8jrF66a7r6i"),
        protocol_token_fee: pubkey!("6kzYo2LMo2q2bkLAD8ienoG5NC1MkNXNTfm8sdyHuX3h"),
        user: payer.pubkey(),
        vault_program: pubkey!("24Uqj9JCLxUeoC3hGfh5W3s9FM9uCHDS2SG3LYwBpyTi"),
        token_program: spl_token::ID,
    };
    let ix = bury(payer.pubkey(), swap);
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn build_payout_ix(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<Instruction, solana_client::client_error::ClientError> {
    let block = get_block(rpc).await?;
    let wagers = get_block_wagers(rpc).await?;

    // Return early if no wagers
    if block.cumulative_sum == 0 || block.reward == 0 {
        return Ok(payout(
            payer.pubkey(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ));
    }

    // Get blockhash
    let solana_block = rpc.get_block(block.ends_at).await?;
    let blockhash = solana_block.blockhash;
    let noise = hashv(&[&block.noise, blockhash.as_ref()]).to_bytes();

    // Calculate the random number.
    let x = u64::from_le_bytes(noise[0..8].try_into().unwrap());
    let y = u64::from_le_bytes(noise[8..16].try_into().unwrap());
    let z = u64::from_le_bytes(noise[16..24].try_into().unwrap());
    let w = u64::from_le_bytes(noise[24..32].try_into().unwrap());
    let roll = (x ^ y ^ z ^ w) % block.cumulative_sum;

    // Find the winning wager
    let mut winner = None;
    for (pubkey, wager) in wagers {
        if roll >= wager.cumulative_sum && roll < wager.cumulative_sum + wager.amount {
            println!("Roll: {}, Winner: {:?}", roll, pubkey);
            winner = Some((pubkey, wager));
            break;
        }
    }

    // Build payout instruction
    let ix = if let Some((pubkey, wager)) = winner {
        payout(
            payer.pubkey(),
            pubkey,
            spl_associated_token_account::get_associated_token_address(
                &wager.authority,
                &spl_token::native_mint::ID,
            ),
        )
    } else {
        payout(payer.pubkey(), Pubkey::new_unique(), Pubkey::new_unique())
    };

    Ok(ix)
}

async fn close_all_wagers(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), solana_client::client_error::ClientError> {
    let block = get_block(rpc).await?;
    let wagers = get_my_wagers(rpc, payer).await?;
    let mut ixs = vec![];
    for (pubkey, wager) in wagers {
        if wager.round != block.current_round {
            let ix = ore_api::sdk::close(payer.pubkey(), pubkey);
            ixs.push(ix);
        }
    }
    submit_transaction(rpc, payer, &ixs).await?;
    Ok(())
}

async fn get_block(rpc: &RpcClient) -> Result<Block, solana_client::client_error::ClientError> {
    let block_pda = ore_api::state::block_pda();
    let account = rpc.get_account(&block_pda.0).await?;
    let block = Block::try_from_bytes(&account.data).unwrap();
    Ok(*block)
}

async fn get_clock(rpc: &RpcClient) -> Result<Clock, solana_client::client_error::ClientError> {
    let data = rpc
        .get_account_data(&solana_sdk::sysvar::clock::ID)
        .await
        .unwrap();
    let clock = bincode::deserialize::<Clock>(&data).unwrap();
    Ok(clock)
}

async fn get_block_wagers(
    rpc: &RpcClient,
) -> Result<Vec<(Pubkey, Wager)>, solana_client::client_error::ClientError> {
    let block = get_block(rpc).await?;
    let filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        56,
        &block.current_round.to_le_bytes(),
    ));
    let wagers = get_program_accounts::<Wager>(rpc, ore_api::ID, vec![filter])
        .await
        .unwrap();
    Ok(wagers)
}

async fn get_my_wagers(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<Vec<(Pubkey, Wager)>, solana_client::client_error::ClientError> {
    let filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        16,
        &payer.pubkey().to_bytes().as_ref(),
    ));
    let wagers = get_program_accounts::<Wager>(rpc, ore_api::ID, vec![filter])
        .await
        .unwrap();
    Ok(wagers)
}

fn generate_seed(payer: &solana_sdk::signer::keypair::Keypair, block: &Block) -> [u8; 32] {
    solana_sdk::hash::hash(
        &[
            payer.pubkey().to_bytes().as_ref(),
            block.current_round.to_le_bytes().as_ref(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_le_bytes()
                .as_ref(),
        ]
        .concat(),
    )
    .to_bytes()
}

async fn submit_transaction(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
    instructions: &[solana_sdk::instruction::Instruction],
) -> Result<solana_sdk::signature::Signature, solana_client::client_error::ClientError> {
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
            Err(e)
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
