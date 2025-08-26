use ore_api::prelude::*;
use steel::*;

use crate::whitelist::AUTHORIZED_ACCOUNTS;

/// Swap in a hashpower market.
pub fn process_swap(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Swap::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);
    let direction = SwapDirection::try_from(args.direction).unwrap();
    let precision = SwapPrecision::try_from(args.precision).unwrap();

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, config_info, fee_collector_info, market_info, miner_info, mint_info, tokens_info, vault_info, system_program, token_program, associated_token_program, ore_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;

    if !AUTHORIZED_ACCOUNTS.contains(signer_info.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    let block: &mut Block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| b.start_slot <= clock.slot)? // Block has started
        .assert_mut(|b| b.end_slot > clock.slot)?; // Block has not ended
    let config = config_info.as_account_mut::<Config>(&ore_api::ID)?;
    fee_collector_info
        .is_writable()?
        .has_address(&config.fee_collector)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.block_id == block.id)?
        .assert_mut(|m| m.base.liquidity() > 0)?
        .assert_mut(|m| m.quote.liquidity() > 0)?;
    mint_info
        .has_address(&market.quote.mint)?
        .has_address(&MINT_ADDRESS)?
        .as_mint()?;
    vault_info
        .is_writable()?
        .as_associated_token_account(market_info.key, mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Load miner.
    let miner = if miner_info.data_is_empty() {
        create_program_account::<Miner>(
            miner_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[MINER, &signer_info.key.to_bytes()],
        )?;
        let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
        miner.authority = *signer_info.key;
        miner.block_id = block.id;
        miner.hashpower = 0;
        miner.seed = args.seed;
        miner.total_hashpower = 0;
        miner.total_rewards = 0;
        miner
    } else {
        miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.authority == *signer_info.key)?
            .assert_mut(|m| m.block_id <= block.id)?
    };

    // Reset miner.
    if miner.block_id != block.id {
        miner.block_id = block.id;
        miner.hashpower = 0;
        miner.seed = args.seed;
    }

    // Pay swap fee.
    if config.fee_rate > 0 {
        fee_collector_info.collect(config.fee_rate, &signer_info)?;
    }

    // Load token acccounts.
    if tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            signer_info,
            tokens_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        tokens_info
            .is_writable()?
            .as_associated_token_account(signer_info.key, mint_info.key)?;
    }

    // Set the sniper fee based on time since the market began.
    let fee_rate = calculate_sniper_fee(block, &clock, config);
    market.fee.rate = fee_rate;

    // Execute the swap
    let mut swap_event = market.swap(amount, direction, precision, clock)?;
    swap_event.authority = *signer_info.key;
    swap_event.block_id = block.id;

    // Transfer tokens
    match direction {
        SwapDirection::Buy => {
            // Update hashpower.
            miner.hashpower += swap_event.base_to_transfer;
            miner.total_hashpower += swap_event.base_to_transfer;
            block.total_hashpower += swap_event.base_to_transfer;

            // TODO if base to transfer is 0, then fail

            // Transfer ORE from signer to market.
            transfer(
                signer_info,
                tokens_info,
                vault_info,
                token_program,
                swap_event.quote_to_transfer,
            )?;
        }
        SwapDirection::Sell => {
            // Update hashpower.
            miner.hashpower -= swap_event.base_to_transfer;
            miner.total_hashpower -= swap_event.base_to_transfer;
            block.total_hashpower -= swap_event.base_to_transfer;

            // TODO if quote to transfer is 0, then fail

            // Transfer ORE from market to signer.
            transfer_signed(
                market_info,
                vault_info,
                tokens_info,
                token_program,
                swap_event.quote_to_transfer,
                &[MARKET],
            )?;
        }
    };

    // Validate vault reserves.
    let vault = vault_info.as_token_account()?;
    market.check_quote_vault(&vault)?;

    // Update block reward.
    let block_reward = vault.amount();
    block.reward = block_reward;

    // Update swap event hashpower.
    swap_event.miner_hashpower = miner.hashpower;
    swap_event.block_hashpower = block.total_hashpower;

    // Emit event.
    program_log(
        &[market_info.clone(), ore_program.clone()],
        &swap_event.to_bytes(),
    )?;

    Ok(())
}

fn calculate_sniper_fee(block: &Block, clock: &Clock, config: &Config) -> u64 {
    let elapsed_slots = clock.slot.saturating_sub(block.start_slot);
    if elapsed_slots >= config.sniper_fee_duration {
        return 0;
    }
    let fee_bps = 5000 * (config.sniper_fee_duration - elapsed_slots) / config.sniper_fee_duration;
    fee_bps
}

#[test]
fn test_sniper_fees() {
    let config = Config {
        sniper_fee_duration: 100,
        fee_rate: 0,
        fee_collector: Pubkey::default(),
        admin: Pubkey::default(),
        block_duration: 0,
    };

    let mut clock = Clock {
        slot: 0,
        epoch_start_timestamp: 0,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: 0,
    };

    let block = Block {
        id: 0,
        opener: Pubkey::default(),
        reward: 0,
        best_hash: [0; 32],
        best_hash_miner: Pubkey::default(),
        start_at: 0,
        start_slot: 0,
        end_slot: u64::MAX,
        slot_hash: [0; 32],
        total_hashpower: 0,
    };

    for i in 0..200 {
        clock.slot = i;
        let fee = calculate_sniper_fee(&block, &clock, &config);
        println!("Slot {}: {} bps fee", i, fee);
    }

    // assert!(false);
}
