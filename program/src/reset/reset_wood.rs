use coal_api::{
    consts::*,
    loaders::*,
    state::{Bus, WoodConfig},
};
use coal_utils::AccountDeserialize;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, msg, program_error::ProgramError, sysvar::Sysvar
};

use crate::calculate_new_reward_rate;

pub fn process_reset_wood<'a, 'info>(accounts: &'a [AccountInfo<'info>], _data: &[u8]) -> ProgramResult {
    msg!("Processing reset for WOOD");
    // Load accounts.
    let [signer, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, mint_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_wood_bus(bus_0_info, 0, true)?;
    load_wood_bus(bus_1_info, 1, true)?;
    load_wood_bus(bus_2_info, 2, true)?;
    load_wood_bus(bus_3_info, 3, true)?;
    load_wood_bus(bus_4_info, 4, true)?;
    load_wood_bus(bus_5_info, 5, true)?;
    load_wood_bus(bus_6_info, 6, true)?;
    load_wood_bus(bus_7_info, 7, true)?;
    msg!("Loaded WOOD bus accounts");
    load_wood_config(config_info, true)?;
    msg!("Loaded WOOD config account");
    load_mint(mint_info, WOOD_MINT_ADDRESS, true)?;
    msg!("Loaded WOOD mint account");
    load_treasury(treasury_info, true)?;
    msg!("Loaded WOOD treasury account");
    load_wood_treasury_tokens(treasury_tokens_info, true)?;
    msg!("Loaded WOOD treasury tokens account");
    load_program(token_program, spl_token::id())?;
    msg!("Loaded WOOD treasury tokens account");
    let busses: [&AccountInfo; BUS_COUNT] = [
        bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info,
        bus_7_info,
    ];

    // Validate enough time has passed since the last reset.
    let mut config_data = config_info.data.borrow_mut();
    let config = WoodConfig::try_from_bytes_mut(&mut config_data)?;
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    if config
        .last_reset_at
        .saturating_add(WOOD_EPOCH_DURATION)
        .gt(&clock.unix_timestamp)
    {
        return Ok(());
    }

    // Update timestamp.
    config.last_reset_at = clock.unix_timestamp;

    // Reset bus accounts and calculate reward rates for next epoch.
    let mut top_balance: u64 = 0u64;
    let mut total_remaining_rewards = 0u64;
    let mut total_theoretical_rewards = 0u64;
    let mut next_epoch_rewards = 0u64;
    
    for i in 0..BUS_COUNT {
        // Parse bus account.
        let mut bus_data = busses[i].data.borrow_mut();
        let bus = Bus::try_from_bytes_mut(&mut bus_data)?;

        // Track top balance.
        if bus.top_balance.gt(&top_balance) {
            top_balance = bus.top_balance;
        }

        total_remaining_rewards = total_remaining_rewards.saturating_add(bus.rewards);
        total_theoretical_rewards = total_theoretical_rewards.saturating_add(bus.theoretical_rewards);

        // Bus rewards grow by 5% each epoch.
        bus.rewards = bus.rewards.saturating_add(bus.rewards.saturating_div(WOOD_PROPOGATION_RATE)).max(1);
        next_epoch_rewards = next_epoch_rewards.saturating_add(bus.rewards);
    }

    let total_epoch_rewards = config.total_epoch_rewards.saturating_sub(total_remaining_rewards).max(0);
    let next_epoch_bus_rewards = next_epoch_rewards.saturating_div(BUS_COUNT as u64);

    // Update global top balance.
    config.top_balance = top_balance;

    // Update the rewards for the next epoch.
    config.total_epoch_rewards = next_epoch_rewards;

    msg!("Total remaining rewards: {}", total_remaining_rewards);
    msg!("Total theoretical rewards: {}", total_theoretical_rewards);
    msg!("Next epoch rewards: {}", next_epoch_rewards);
    msg!("Next epoch bus rewards: {}", next_epoch_bus_rewards);
    // Update base reward rate for next epoch.
    config.base_reward_rate =
        calculate_new_reward_rate(config.base_reward_rate, total_theoretical_rewards, next_epoch_rewards, next_epoch_bus_rewards);
    msg!("New base reward rate: {}", config.base_reward_rate);

    // If base reward rate is too low, increment min difficulty by 1 and double base reward rate.
    if config.base_reward_rate.le(&BASE_WOOD_REWARD_RATE_MIN_THRESHOLD) {
        config.min_difficulty = config.min_difficulty.checked_add(1).unwrap();
        config.base_reward_rate = config.base_reward_rate.checked_mul(2).unwrap();
    }

    // If base reward rate is too high, decrement min difficulty by 1 and halve base reward rate.
    if config.base_reward_rate.ge(&BASE_WOOD_REWARD_RATE_MAX_THRESHOLD) && config.min_difficulty.gt(&1) {
        config.min_difficulty = config.min_difficulty.checked_sub(1).unwrap();
        config.base_reward_rate = config.base_reward_rate.checked_div(2).unwrap();
    }

    
    // Fund the treasury token account.
    solana_program::program::invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            mint_info.key,
            treasury_tokens_info.key,
            treasury_info.key,
            &[treasury_info.key],
            total_epoch_rewards,
        )?,
        &[
            token_program.clone(),
            mint_info.clone(),
            treasury_tokens_info.clone(),
            treasury_info.clone(),
        ],
        &[&[TREASURY, &[TREASURY_BUMP]]],
    )?;

    Ok(())
}