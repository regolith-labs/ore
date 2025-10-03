use std::{collections::HashMap, str::FromStr};

use meteora_pools_sdk::accounts::Pool;
use meteora_vault_sdk::accounts::Vault;
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
    native_token::lamports_to_sol,
    pubkey,
    pubkey::Pubkey,
    signature::{read_keypair_file, Signer},
    slot_hashes::SlotHashes,
    transaction::Transaction,
};
use spl_associated_token_account::get_associated_token_address;
use spl_token::{amount_to_ui_amount, ui_amount_to_amount};
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
        "automations" => {
            log_automations(&rpc).await.unwrap();
        }
        "boost" => {
            boost(&rpc, &payer).await.unwrap();
        }
        "clock" => {
            log_clock(&rpc).await.unwrap();
        }
        "claim" => {
            claim(&rpc, &payer).await.unwrap();
        }
        "board" => {
            log_board(&rpc).await.unwrap();
        }
        "config" => {
            log_config(&rpc).await.unwrap();
        }
        "initialize" => {
            initialize(&rpc, &payer).await.unwrap();
        }
        "bury" => {
            bury(&rpc, &payer).await.unwrap();
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
        "pool" => {
            log_meteora_pool(&rpc).await.unwrap();
        }
        "deploy" => {
            deploy(&rpc, &payer).await.unwrap();
        }
        "stake" => {
            log_stake(&rpc, &payer).await.unwrap();
        }
        "deploy_all" => {
            deploy_all(&rpc, &payer).await.unwrap();
        }
        "round" => {
            log_round(&rpc).await.unwrap();
        }
        "seeker" => {
            log_seeker(&rpc).await.unwrap();
        }
        "set_admin" => {
            set_admin(&rpc, &payer).await.unwrap();
        }
        "set_fee_collector" => {
            set_fee_collector(&rpc, &payer).await.unwrap();
        }
        "ata" => {
            ata(&rpc, &payer).await.unwrap();
        }
        "checkpoint_all" => {
            checkpoint_all(&rpc, &payer).await.unwrap();
        }
        "claim_seeker" => {
            claim_seeker(&rpc, &payer).await.unwrap();
        }
        "participating_miners" => {
            participating_miners(&rpc).await.unwrap();
        }
        "keys" => {
            keys().await.unwrap();
        }
        _ => panic!("Invalid command"),
    };
}

async fn participating_miners(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let round_id = std::env::var("ID").expect("Missing ID env var");
    let round_id = u64::from_str(&round_id).expect("Invalid ID");
    let miners = get_miners_participating(rpc, round_id).await?;
    for (i, (address, miner)) in miners.iter().enumerate() {
        println!("{}: {}", i, miner.authority);
    }
    Ok(())
}

async fn log_stake(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let authority = std::env::var("AUTHORITY").unwrap_or(payer.pubkey().to_string());
    let authority = Pubkey::from_str(&authority).expect("Invalid AUTHORITY");
    let staker_address = ore_api::state::stake_pda(authority).0;
    let stake = get_stake(rpc, authority).await?;
    println!("Stake");
    println!("  address: {}", staker_address);
    println!("  authority: {}", authority);
    println!("  balance: {}", stake.balance);
    println!("  last_claim_at: {}", stake.last_claim_at);
    println!("  last_deposit_at: {}", stake.last_deposit_at);
    println!("  last_withdraw_at: {}", stake.last_withdraw_at);
    println!(
        "  rewards_factor: {}",
        stake.rewards_factor.to_i80f48().to_string()
    );
    println!("  rewards: {}", stake.rewards);
    println!("  lifetime_rewards: {}", stake.lifetime_rewards);
    println!("  is_seeker: {}", stake.is_seeker);

    Ok(())
}

async fn ata(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let user = pubkey!("45db2FSR4mcXdSVVZbKbwojU6uYDpMyhpEi7cC8nHaWG");
    let token = pubkey!("So11111111111111111111111111111111111111112");
    let ata = get_associated_token_address(&user, &token);
    let ix = spl_associated_token_account::instruction::create_associated_token_account(
        &payer.pubkey(),
        &user,
        &token,
        &spl_token::ID,
    );
    submit_transaction(rpc, payer, &[ix]).await?;
    let account = rpc.get_account(&ata).await?;
    println!("ATA: {}", ata);
    println!("Account: {:?}", account);
    Ok(())
}

async fn boost(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix = ore_api::sdk::boost(payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn keys() -> Result<(), anyhow::Error> {
    let treasury_address = ore_api::state::treasury_pda().0;
    let config_address = ore_api::state::config_pda().0;
    let board_address = ore_api::state::board_pda().0;
    let address = pubkey!("pqspJ298ryBjazPAr95J9sULCVpZe3HbZTWkbC1zrkS");
    let miner_address = ore_api::state::miner_pda(address).0;
    println!("Treasury: {}", treasury_address);
    println!("Config: {}", config_address);
    println!("Board: {}", board_address);
    println!("Miner: {}", miner_address);

    Ok(())
}

async fn initialize(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix = ore_api::sdk::initialize(payer.pubkey());
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn claim(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let ix_sol = ore_api::sdk::claim_sol(payer.pubkey(), u64::MAX);
    let ix_ore = ore_api::sdk::claim_ore(payer.pubkey(), u64::MAX);
    submit_transaction(rpc, payer, &[ix_sol, ix_ore]).await?;
    Ok(())
}

async fn bury(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let amount_str = std::env::var("AMOUNT").expect("Missing AMOUNT env var");
    let amount_f64 = f64::from_str(&amount_str).expect("Invalid AMOUNT");
    let amount_u64 = ui_amount_to_amount(amount_f64, TOKEN_DECIMALS);
    let wrap_ix = ore_api::sdk::wrap(payer.pubkey());
    let bury_ix = ore_api::sdk::bury(payer.pubkey(), amount_u64);
    // submit_transaction(rpc, payer, &[wrap_ix, bury_ix]).await?;
    simulate_transaction(rpc, payer, &[wrap_ix, bury_ix]).await;
    Ok(())
}

async fn reset(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let board = get_board(rpc).await?;
    let config = get_config(rpc).await?;
    let slot_hashes = get_slot_hashes(rpc).await?;
    if let Some(slot_hash) = slot_hashes.get(&board.end_slot) {
        let id = get_winning_square(&slot_hash.to_bytes());
        // let square = get_square(rpc).await?;
        println!("Winning square: {}", id);
        // println!("Miners: {:?}", square.miners);
        // miners = square.miners[id as usize].to_vec();
    };
    let reset_ix = ore_api::sdk::reset(
        payer.pubkey(),
        config.fee_collector,
        board.round_id,
        Pubkey::default(),
    );
    // simulate_transaction(rpc, payer, &[reset_ix]).await;
    submit_transaction(rpc, payer, &[reset_ix]).await?;
    Ok(())
}

async fn deploy(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let amount = std::env::var("AMOUNT").expect("Missing AMOUNT env var");
    let amount = u64::from_str(&amount).expect("Invalid AMOUNT");
    let square_id = std::env::var("SQUARE").expect("Missing SQUARE env var");
    let square_id = u64::from_str(&square_id).expect("Invalid SQUARE");
    let board = get_board(rpc).await?;
    let mut squares = [false; 25];
    squares[square_id as usize] = true;
    let ix = ore_api::sdk::deploy(
        payer.pubkey(),
        payer.pubkey(),
        amount,
        board.round_id,
        squares,
    );
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn deploy_all(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let amount = std::env::var("AMOUNT").expect("Missing AMOUNT env var");
    let amount = u64::from_str(&amount).expect("Invalid AMOUNT");
    let board = get_board(rpc).await?;
    let squares = [true; 25];
    let ix = ore_api::sdk::deploy(
        payer.pubkey(),
        payer.pubkey(),
        board.round_id,
        amount,
        squares,
    );
    submit_transaction(rpc, payer, &[ix]).await?;
    Ok(())
}

async fn claim_seeker(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let seeker_mint = pubkey!("5mXbkqKz883aufhAsx3p5Z1NcvD2ppZbdTTznM6oUKLj");
    let ix = ore_api::sdk::claim_seeker(payer.pubkey(), seeker_mint);
    // submit_transaction(rpc, payer, &[ix]).await?;
    simulate_transaction(rpc, payer, &[ix]).await;
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

async fn checkpoint_all(
    rpc: &RpcClient,
    payer: &solana_sdk::signer::keypair::Keypair,
) -> Result<(), anyhow::Error> {
    let clock = get_clock(rpc).await?;
    let miners = get_miners(rpc).await?;
    let mut expiry_slots = HashMap::new();
    let mut ixs = vec![];
    for (i, (_address, miner)) in miners.iter().enumerate() {
        if miner.checkpoint_id < miner.round_id {
            // Log the expiry slot for the round.
            if !expiry_slots.contains_key(&miner.round_id) {
                if let Ok(round) = get_round(rpc, miner.round_id).await {
                    expiry_slots.insert(miner.round_id, round.expires_at);
                }
            }

            // Get the expiry slot for the round.
            let Some(expires_at) = expiry_slots.get(&miner.round_id) else {
                continue;
            };

            println!(
                "[{}/{}] Checkpoint miner: {} ({} s)",
                i + 1,
                miners.len(),
                miner.authority,
                (expires_at - clock.slot) as f64 * 0.4
            );

            // If we are in fee collection period, checkpoint the miner.
            if clock.slot >= expires_at - TWELVE_HOURS_SLOTS {
                ixs.push(ore_api::sdk::checkpoint(
                    payer.pubkey(),
                    miner.authority,
                    miner.round_id,
                ));
            }

            // If we have more than 10 ixs, submit them.
            if ixs.len() >= 10 {
                submit_transaction(rpc, payer, &ixs).await?;
                ixs.clear();
            }
        }
    }

    // Submit the remaining ixs.
    if !ixs.is_empty() {
        submit_transaction(rpc, payer, &ixs).await?;
    }

    Ok(())
}

async fn log_meteora_pool(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let address = pubkey!("GgaDTFbqdgjoZz3FP7zrtofGwnRS4E6MCzmmD5Ni1Mxj");
    let pool = get_meteora_pool(rpc, address).await?;
    let vault_a = get_meteora_vault(rpc, pool.a_vault).await?;
    let vault_b = get_meteora_vault(rpc, pool.b_vault).await?;

    println!("Pool");
    println!("  address: {}", address);
    println!("  lp_mint: {}", pool.lp_mint);
    println!("  token_a_mint: {}", pool.token_a_mint);
    println!("  token_b_mint: {}", pool.token_b_mint);
    println!("  a_vault: {}", pool.a_vault);
    println!("  b_vault: {}", pool.b_vault);
    println!("  a_token_vault: {}", vault_a.token_vault);
    println!("  b_token_vault: {}", vault_b.token_vault);
    println!("  a_vault_lp_mint: {}", vault_a.lp_mint);
    println!("  b_vault_lp_mint: {}", vault_b.lp_mint);
    println!("  a_vault_lp: {}", pool.a_vault_lp);
    println!("  b_vault_lp: {}", pool.b_vault_lp);
    println!("  protocol_token_fee: {}", pool.protocol_token_b_fee);

    // pool: *pool.key,
    // user_source_token: *user_source_token.key,
    // user_destination_token: *user_destination_token.key,
    // a_vault: *a_vault.key,
    // b_vault: *b_vault.key,
    // a_token_vault: *a_token_vault.key,
    // b_token_vault: *b_token_vault.key,
    // a_vault_lp_mint: *a_vault_lp_mint.key,
    // b_vault_lp_mint: *b_vault_lp_mint.key,
    // a_vault_lp: *a_vault_lp.key,
    // b_vault_lp: *b_vault_lp.key,
    // protocol_token_fee: *protocol_token_fee.key,
    // user: *user.key,
    // vault_program: *vault_program.key,
    // token_program: *token_program.key,

    Ok(())
}

async fn log_automations(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let automations = get_automations(rpc).await?;
    for (i, (address, automation)) in automations.iter().enumerate() {
        println!("[{}/{}] {}", i + 1, automations.len(), address);
        println!("  authority: {}", automation.authority);
        println!("  balance: {}", automation.balance);
        println!("  executor: {}", automation.executor);
        println!("  fee: {}", automation.fee);
        println!("  mask: {}", automation.mask);
        println!("  strategy: {}", automation.strategy);
        println!();
    }
    Ok(())
}

async fn log_treasury(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let treasury_address = ore_api::state::treasury_pda().0;
    let treasury = get_treasury(rpc).await?;
    println!("Treasury");
    println!("  address: {}", treasury_address);
    println!("  balance: {} SOL", lamports_to_sol(treasury.balance));
    println!(
        "  motherlode: {} ORE",
        amount_to_ui_amount(treasury.motherlode, TOKEN_DECIMALS)
    );
    println!(
        "  rewards_factor: {}",
        treasury.rewards_factor.to_i80f48().to_string()
    );
    println!("  total_staked: {}", treasury.total_staked);
    Ok(())
}

async fn log_round(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let id = std::env::var("ID").expect("Missing ID env var");
    let id = u64::from_str(&id).expect("Invalid ID");
    let round_address = round_pda(id).0;
    let round = get_round(rpc, id).await?;
    let rng = round.rng();
    println!("Round");
    println!("  Address: {}", round_address);
    println!("  Count: {:?}", round.count);
    println!("  Deployed: {:?}", round.deployed);
    println!("  Expires at: {}", round.expires_at);
    println!("  Id: {:?}", round.id);
    println!("  Motherlode: {}", round.motherlode);
    println!("  Rent payer: {}", round.rent_payer);
    println!("  Slot hash: {:?}", round.slot_hash);
    println!("  Top miner: {:?}", round.top_miner);
    println!("  Top miner reward: {}", round.top_miner_reward);
    println!("  Total deployed: {}", round.total_deployed);
    println!("  Total vaulted: {}", round.total_vaulted);
    println!("  Total winnings: {}", round.total_winnings);
    if let Some(rng) = rng {
        println!("  Winning square: {}", round.winning_square(rng));
    }
    // if round.slot_hash != [0; 32] {
    //     println!("  Winning square: {}", get_winning_square(&round.slot_hash));
    // }
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
    println!("  deployed: {:?}", miner.deployed);
    println!("  cumulative: {:?}", miner.cumulative);
    println!("  rewards_sol: {}", miner.rewards_sol);
    println!("  rewards_ore: {}", miner.rewards_ore);
    println!("  round_id: {}", miner.round_id);
    println!("  checkpoint_id: {}", miner.checkpoint_id);
    println!("  lifetime_rewards_sol: {}", miner.lifetime_rewards_sol);
    println!("  lifetime_rewards_ore: {}", miner.lifetime_rewards_ore);
    Ok(())
}

async fn log_seeker(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let mint = std::env::var("MINT").unwrap();
    let mint = Pubkey::from_str(&mint).expect("Invalid MINT");
    let seeker = get_seeker(&rpc, mint).await?;
    let seeker_address = ore_api::state::seeker_pda(mint).0;
    println!("Seeker");
    println!("  address: {}", seeker_address);
    println!("  mint: {}", seeker.mint);
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
    println!("  last_boost: {}", config.last_boost);
    println!("  fee_collector: {}", config.fee_collector);
    Ok(())
}

async fn log_board(rpc: &RpcClient) -> Result<(), anyhow::Error> {
    let board = get_board(&rpc).await?;
    let clock = get_clock(&rpc).await?;
    print_board(board, &clock);
    Ok(())
}

fn print_board(board: Board, clock: &Clock) {
    let current_slot = clock.slot;
    println!("Board");
    println!("  Id: {:?}", board.round_id);
    println!("  Start slot: {}", board.start_slot);
    println!("  End slot: {}", board.end_slot);
    println!(
        "  Time remaining: {} sec",
        (board.end_slot.saturating_sub(current_slot) as f64) * 0.4
    );
}

async fn get_automations(rpc: &RpcClient) -> Result<Vec<(Pubkey, Automation)>, anyhow::Error> {
    const REGOLITH_EXECUTOR: Pubkey = pubkey!("HNWhK5f8RMWBqcA7mXJPaxdTPGrha3rrqUrri7HSKb3T");
    let filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(
        56,
        &REGOLITH_EXECUTOR.to_bytes(),
    ));
    let automations = get_program_accounts::<Automation>(rpc, ore_api::ID, vec![filter]).await?;
    Ok(automations)
}

async fn get_meteora_pool(rpc: &RpcClient, address: Pubkey) -> Result<Pool, anyhow::Error> {
    let data = rpc.get_account_data(&address).await?;
    let pool = Pool::from_bytes(&data)?;
    Ok(pool)
}

async fn get_meteora_vault(rpc: &RpcClient, address: Pubkey) -> Result<Vault, anyhow::Error> {
    let data = rpc.get_account_data(&address).await?;
    let vault = Vault::from_bytes(&data)?;
    Ok(vault)
}

async fn get_board(rpc: &RpcClient) -> Result<Board, anyhow::Error> {
    let board_pda = ore_api::state::board_pda();
    let account = rpc.get_account(&board_pda.0).await?;
    let board = Board::try_from_bytes(&account.data)?;
    Ok(*board)
}

async fn get_slot_hashes(rpc: &RpcClient) -> Result<SlotHashes, anyhow::Error> {
    let data = rpc
        .get_account_data(&solana_sdk::sysvar::slot_hashes::ID)
        .await?;
    let slot_hashes = bincode::deserialize::<SlotHashes>(&data)?;
    Ok(slot_hashes)
}

async fn get_round(rpc: &RpcClient, id: u64) -> Result<Round, anyhow::Error> {
    let round_pda = ore_api::state::round_pda(id);
    let account = rpc.get_account(&round_pda.0).await?;
    let round = Round::try_from_bytes(&account.data)?;
    Ok(*round)
}

async fn get_treasury(rpc: &RpcClient) -> Result<Treasury, anyhow::Error> {
    let treasury_pda = ore_api::state::treasury_pda();
    let account = rpc.get_account(&treasury_pda.0).await?;
    let treasury = Treasury::try_from_bytes(&account.data)?;
    Ok(*treasury)
}

async fn get_config(rpc: &RpcClient) -> Result<Config, anyhow::Error> {
    let config_pda = ore_api::state::config_pda();
    let account = rpc.get_account(&config_pda.0).await?;
    let config = Config::try_from_bytes(&account.data)?;
    Ok(*config)
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

async fn get_seeker(rpc: &RpcClient, mint: Pubkey) -> Result<Seeker, anyhow::Error> {
    let seeker_pda = ore_api::state::seeker_pda(mint);
    let account = rpc.get_account(&seeker_pda.0).await?;
    let seeker = Seeker::try_from_bytes(&account.data)?;
    Ok(*seeker)
}

async fn get_stake(rpc: &RpcClient, authority: Pubkey) -> Result<Stake, anyhow::Error> {
    let stake_pda = ore_api::state::stake_pda(authority);
    let account = rpc.get_account(&stake_pda.0).await?;
    let stake = Stake::try_from_bytes(&account.data)?;
    Ok(*stake)
}

#[allow(dead_code)]
async fn get_miners(rpc: &RpcClient) -> Result<Vec<(Pubkey, Miner)>, anyhow::Error> {
    let miners = get_program_accounts::<Miner>(rpc, ore_api::ID, vec![]).await?;
    Ok(miners)
}

async fn get_miners_old(rpc: &RpcClient) -> Result<Vec<(Pubkey, MinerOLD)>, anyhow::Error> {
    let miners = get_program_accounts::<MinerOLD>(rpc, ore_api::ID, vec![]).await?;
    Ok(miners)
}

async fn get_miners_participating(
    rpc: &RpcClient,
    round_id: u64,
) -> Result<Vec<(Pubkey, Miner)>, anyhow::Error> {
    let filter = RpcFilterType::Memcmp(Memcmp::new_base58_encoded(472, &round_id.to_le_bytes()));
    let miners = get_program_accounts::<Miner>(rpc, ore_api::ID, vec![filter]).await?;
    Ok(miners)
}

fn get_winning_square(slot_hash: &[u8]) -> u64 {
    // Use slot hash to generate a random u64
    let r1 = u64::from_le_bytes(slot_hash[0..8].try_into().unwrap());
    let r2 = u64::from_le_bytes(slot_hash[8..16].try_into().unwrap());
    let r3 = u64::from_le_bytes(slot_hash[16..24].try_into().unwrap());
    let r4 = u64::from_le_bytes(slot_hash[24..32].try_into().unwrap());
    let r = r1 ^ r2 ^ r3 ^ r4;

    // Returns a value in the range [0, 24] inclusive
    r % 25
}

#[allow(dead_code)]
async fn simulate_transaction(
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

async fn submit_transaction_no_confirm(
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

    match rpc.send_transaction(&transaction).await {
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
