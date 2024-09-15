use coal_api::{
    consts::*,
    error::CoalError,
    loaders::*,
    state::{Config, Bus},
};
use coal_utils::AccountDeserialize;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult, program_error::ProgramError, program_pack::Pack, sysvar::Sysvar
};
use spl_token::state::Mint;

use crate::calculate_new_reward_rate;

/// Reset tops up the bus balances, updates the base reward rate, and sets up the ORE program for the next epoch.
pub fn process_reset_coal<'a, 'info>(accounts: &'a [AccountInfo<'info>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, mint_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_coal_bus(bus_0_info, 0, true)?;
    load_coal_bus(bus_1_info, 1, true)?;
    load_coal_bus(bus_2_info, 2, true)?;
    load_coal_bus(bus_3_info, 3, true)?;
    load_coal_bus(bus_4_info, 4, true)?;
    load_coal_bus(bus_5_info, 5, true)?;
    load_coal_bus(bus_6_info, 6, true)?;
    load_coal_bus(bus_7_info, 7, true)?;
    load_coal_config(config_info, true)?;
    load_mint(mint_info, COAL_MINT_ADDRESS, true)?;
    load_treasury(treasury_info, true)?;
    load_coal_treasury_tokens(treasury_tokens_info, true)?;
    load_program(token_program, spl_token::id())?;
    let busses: [&AccountInfo; BUS_COUNT] = [
        bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info,
        bus_7_info,
    ];

    // Validate enough time has passed since the last reset.
    let mut config_data = config_info.data.borrow_mut();
    let config = Config::try_from_bytes_mut(&mut config_data)?;
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    if config
        .last_reset_at
        .saturating_add(COAL_EPOCH_DURATION)
        .gt(&clock.unix_timestamp)
    {
        return Ok(());
    }

    // Update timestamp.
    config.last_reset_at = clock.unix_timestamp;

    // Max supply check.
    let mint = Mint::unpack(&mint_info.data.borrow()).expect("Failed to parse mint");
    if mint.supply.ge(&MAX_COAL_SUPPLY) {
        return Err(CoalError::MaxSupply.into());
    }

    // For each 5% of total supply, reduce the BUS_EPOCH_REWARDS and MAX_EPOCH_REWARDS by 50%
    // The halving is done to incentivize the accumulation of the token.
    // Halving should only occur at 5% intervals.
    let supply_percentage = (mint.supply as f64 / MAX_COAL_SUPPLY as f64) * 100.0;
    let halving_factor = 2u64.pow((supply_percentage / 5.0) as u32);
    let adjusted_target_rewards = TARGET_COAL_EPOCH_REWARDS / halving_factor;
    let adjusted_bus_epoch_rewards = BUS_COAL_EPOCH_REWARDS / halving_factor;
    let adjusted_max_epoch_rewards = MAX_COAL_EPOCH_REWARDS / halving_factor;   

    // Reset bus accounts and calculate actual rewards mined since last reset.
    let mut total_remaining_rewards = 0u64;
    let mut total_theoretical_rewards = 0u64;
    let mut top_balance = 0u64;
    for i in 0..BUS_COUNT {
        // Parse bus account.
        let mut bus_data = busses[i].data.borrow_mut();
        let bus = Bus::try_from_bytes_mut(&mut bus_data)?;

        // Track top balance.
        if bus.top_balance.gt(&top_balance) {
            top_balance = bus.top_balance;
        }

        // Track accumulators.
        total_remaining_rewards = total_remaining_rewards.saturating_add(bus.rewards);
        total_theoretical_rewards =
            total_theoretical_rewards.saturating_add(bus.theoretical_rewards);

        // Reset bus account for new epoch.
        bus.rewards = adjusted_bus_epoch_rewards;
        bus.theoretical_rewards = 0;
        bus.top_balance = 0;
    }
    let total_epoch_rewards = adjusted_max_epoch_rewards.saturating_sub(total_remaining_rewards);

    // Update global top balance.
    config.top_balance = top_balance;

    // Update base reward rate for next epoch.
    config.base_reward_rate =
        calculate_new_reward_rate(config.base_reward_rate, total_theoretical_rewards, adjusted_target_rewards, adjusted_bus_epoch_rewards);

    let adjusted_base_reward_threshold = BASE_COAL_REWARD_RATE_MIN_THRESHOLD / halving_factor;
    let adjusted_base_reward_max_threshold = BASE_COAL_REWARD_RATE_MAX_THRESHOLD / halving_factor;
   
    // If base reward rate is too low, increment min difficulty by 1 and double base reward rate.
    if config.base_reward_rate.le(&adjusted_base_reward_threshold) {
        config.min_difficulty = config.min_difficulty.checked_add(1).unwrap();
        config.base_reward_rate = config.base_reward_rate.checked_mul(2).unwrap();
    }

    // If base reward rate is too high, decrement min difficulty by 1 and halve base reward rate.
    if config.base_reward_rate.ge(&adjusted_base_reward_max_threshold) && config.min_difficulty.gt(&1) {
        config.min_difficulty = config.min_difficulty.checked_sub(1).unwrap();
        config.base_reward_rate = config.base_reward_rate.checked_div(2).unwrap();
    }

    
    // Fund the treasury token account.
    let amount = MAX_COAL_SUPPLY
        .saturating_sub(mint.supply)
        .min(total_epoch_rewards);
    solana_program::program::invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            mint_info.key,
            treasury_tokens_info.key,
            treasury_info.key,
            &[treasury_info.key],
            amount,
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

#[cfg(test)]
mod tests {
    use rand::{distributions::Uniform, Rng};
    use crate::calculate_new_reward_rate;
    use coal_api::consts::{
        BASE_COAL_REWARD_RATE_MIN_THRESHOLD, BUS_COAL_EPOCH_REWARDS, MAX_COAL_EPOCH_REWARDS, SMOOTHING_FACTOR,
        TARGET_COAL_EPOCH_REWARDS,
    };

    const FUZZ_SIZE: u64 = 10_000;

    #[test]
    fn test_calculate_new_reward_rate_target() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, TARGET_COAL_EPOCH_REWARDS, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_div_by_zero() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 0, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }   

    #[test]
    fn test_calculate_new_reward_rate_lower() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_COAL_EPOCH_REWARDS.saturating_add(1_000_000_000),
            TARGET_COAL_EPOCH_REWARDS,
            BUS_COAL_EPOCH_REWARDS
        );
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower_edge() {
        let current_rate = BASE_COAL_REWARD_RATE_MIN_THRESHOLD;
        let new_rate = calculate_new_reward_rate(current_rate, TARGET_COAL_EPOCH_REWARDS + 1, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, BUS_COAL_EPOCH_REWARDS));
            let actual_rewards: u64 =
                rng.sample(Uniform::new(TARGET_COAL_EPOCH_REWARDS, MAX_COAL_EPOCH_REWARDS));
            let new_rate = calculate_new_reward_rate(current_rate, actual_rewards, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
            assert!(new_rate.lt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_higher() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_COAL_EPOCH_REWARDS.saturating_sub(1_000_000_000_000),
            TARGET_COAL_EPOCH_REWARDS,
            BUS_COAL_EPOCH_REWARDS
        );
        assert!(new_rate.gt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_higher_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, BUS_COAL_EPOCH_REWARDS));
            let actual_rewards: u64 = rng.sample(Uniform::new(1, TARGET_COAL_EPOCH_REWARDS));
            let new_rate = calculate_new_reward_rate(current_rate, actual_rewards, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
            assert!(new_rate.gt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_max_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 1, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate.saturating_mul(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, u64::MAX, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_max_inputs() {
        let new_rate = calculate_new_reward_rate(BUS_COAL_EPOCH_REWARDS, MAX_COAL_EPOCH_REWARDS, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
        assert!(new_rate.eq(&BUS_COAL_EPOCH_REWARDS.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_inputs() {
        let new_rate = calculate_new_reward_rate(1, 1, TARGET_COAL_EPOCH_REWARDS, BUS_COAL_EPOCH_REWARDS);
        assert!(new_rate.eq(&1u64.saturating_mul(SMOOTHING_FACTOR)));
    }
}
