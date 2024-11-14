use ore_api::prelude::*;
use steel::*;

/// Reset tops up the bus balances, updates the base reward rate, and sets up the ORE program for the next epoch.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, mint_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let bus_0 = bus_0_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 0)?;
    let bus_1 = bus_1_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 1)?;
    let bus_2 = bus_2_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 2)?;
    let bus_3 = bus_3_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 3)?;
    let bus_4 = bus_4_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 4)?;
    let bus_5 = bus_5_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 5)?;
    let bus_6 = bus_6_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 6)?;
    let bus_7 = bus_7_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 7)?;
    let config = config_info
        .is_config()?
        .as_account_mut::<Config>(&ore_api::ID)?;
    let mint = mint_info
        .has_address(&MINT_ADDRESS)?
        .is_writable()?
        .as_mint()?;
    treasury_info.is_treasury()?.is_writable()?;
    treasury_tokens_info.is_treasury_tokens()?.is_writable()?;
    token_program.is_program(&spl_token::ID)?;

    // Max supply check.
    if mint.supply.ge(&MAX_SUPPLY) {
        return Err(OreError::MaxSupply.into());
    }

    // Validate enough time has passed since the last reset.
    let clock = Clock::get()?;
    if config
        .last_reset_at
        .saturating_add(EPOCH_DURATION)
        .gt(&clock.unix_timestamp)
    {
        return Ok(());
    }

    // Update timestamp.
    config.last_reset_at = clock.unix_timestamp;

    // Calculate bus rewards
    let target_epoch_rewards = config
        .target_emmissions_rate
        .checked_mul(EPOCH_MINUTES as u64)
        .unwrap();
    let max_epoch_rewards = target_epoch_rewards.checked_mul(BUS_COUNT as u64).unwrap();

    // Reset bus accounts and calculate actual rewards mined since last reset.
    let busses = [bus_0, bus_1, bus_2, bus_3, bus_4, bus_5, bus_6, bus_7];
    let mut total_remaining_rewards = 0u64;
    let mut total_theoretical_rewards = 0u64;
    for bus in busses {
        // Track accumulators.
        total_remaining_rewards = total_remaining_rewards.saturating_add(bus.rewards);
        total_theoretical_rewards =
            total_theoretical_rewards.saturating_add(bus.theoretical_rewards);

        // Reset bus account for new epoch.
        // TODO Top up bus rewards counters.
        // TODO Handle max supply case
        // TODO What if this is not divisible by BUS_COUNT?
        // bus.rewards = target_epoch_rewards;
        bus.theoretical_rewards = 0;
    }
    let total_epoch_rewards = max_epoch_rewards.saturating_sub(total_remaining_rewards);

    // Update base reward rate for next epoch.
    config.base_reward_rate = calculate_new_reward_rate(
        config.base_reward_rate,
        total_theoretical_rewards,
        target_epoch_rewards,
        target_epoch_rewards,
    );

    // If base reward rate is too low, increment min difficulty by 1 and double base reward rate.
    if config.base_reward_rate.le(&BASE_REWARD_RATE_MIN_THRESHOLD) {
        config.min_difficulty = config.min_difficulty.checked_add(1).unwrap();
        config.base_reward_rate = config.base_reward_rate.checked_mul(2).unwrap();
    }

    // If base reward rate is too high, decrement min difficulty by 1 and halve base reward rate.
    if config.base_reward_rate.ge(&BASE_REWARD_RATE_MAX_THRESHOLD) && config.min_difficulty.gt(&1) {
        config.min_difficulty = config.min_difficulty.checked_sub(1).unwrap();
        config.base_reward_rate = config.base_reward_rate.checked_div(2).unwrap();
    }

    // Calculate amount to mint.
    let amount = MAX_SUPPLY
        .saturating_sub(mint.supply)
        .min(total_epoch_rewards);

    // Adjust curve down 90%
    if mint.supply > config.next_emmissions_rate_update {
        config.target_emmissions_rate = config
            .target_emmissions_rate
            .checked_mul(9)
            .unwrap()
            .checked_div(10)
            .unwrap();
        config.next_emmissions_rate_update = config
            .next_emmissions_rate_update
            .checked_add(config.target_emmissions_rate * 60 * 24 * 365 * 2)
            .unwrap();
    }

    // Fund the treasury token account.
    mint_to_signed(
        mint_info,
        treasury_tokens_info,
        treasury_info,
        token_program,
        amount,
        &[TREASURY],
    )?;

    Ok(())
}

/// This function calculates what the new reward rate should be based on how many total rewards
/// were mined in the prior epoch. The math is largely identitical to function used by the Bitcoin
/// network to update the difficulty between each epoch.
///
/// new_rate = current_rate * (target_rewards / actual_rewards)
///
/// The new rate is then smoothed by a constant factor to avoid large fluctuations. In Ore's case,
/// the epochs are short (60 seconds) so a smoothing factor of 2 has been chosen. That is, the reward rate
/// can at most double or halve from one epoch to the next.
pub(crate) fn calculate_new_reward_rate(
    current_rate: u64,
    epoch_rewards: u64,
    target_epoch_rewards: u64,
    bus_epoch_rewards: u64,
) -> u64 {
    // Avoid division by zero. Leave the reward rate unchanged, if detected.
    if epoch_rewards.eq(&0) {
        return current_rate;
    }

    // Calculate new reward rate.
    let new_rate = (current_rate as u128)
        .saturating_mul(target_epoch_rewards as u128)
        .saturating_div(epoch_rewards as u128) as u64;

    // Smooth reward rate so it cannot change by more than a constant factor from one epoch to the next.
    let new_rate_min = current_rate.saturating_div(SMOOTHING_FACTOR);
    let new_rate_max = current_rate.saturating_mul(SMOOTHING_FACTOR);
    let new_rate_smoothed = new_rate.min(new_rate_max).max(new_rate_min);

    // Prevent reward rate from dropping below 1 or exceeding BUS_EPOCH_REWARDS and return.
    new_rate_smoothed.max(1).min(bus_epoch_rewards)
}

pub(crate) fn get_target_emissions_rate(current_supply: u64) -> u64 {
    match current_supply {
        n if n < ONE_ORE * 525_600 => ONE_ORE,
        n if n < ONE_ORE * 998_640 => ONE_ORE,
        n if n < ONE_ORE * 1_424_376 => ONE_ORE,
        n if n < ONE_ORE * 1_807_538 => ONE_ORE,
        n if n < ONE_ORE * 1_424_376 => ONE_ORE,
        n if n < ONE_ORE * 2_152_385 => ONE_ORE,
        n if n < ONE_ORE * 2_462_746 => ONE_ORE,
        n if n < ONE_ORE * 2_742_071 => ONE_ORE,
        n if n < ONE_ORE * 2_993_464 => ONE_ORE,
        n if n < ONE_ORE * 3_219_718 => ONE_ORE,
        n if n < ONE_ORE * 3_423_346 => ONE_ORE,
        n if n < ONE_ORE * 3_606_612 => ONE_ORE,
        n if n < ONE_ORE * 3_771_550 => ONE_ORE,
        n if n < ONE_ORE * 3_919_995 => ONE_ORE,
        n if n < ONE_ORE * 4_053_596 => ONE_ORE,
        n if n < ONE_ORE * 4_173_836 => ONE_ORE,
        n if n < ONE_ORE * 4_282_053 => ONE_ORE,
        n if n < ONE_ORE * 4_379_447 => ONE_ORE,
        n if n < ONE_ORE * 4_467_103 => ONE_ORE,
        n if n < ONE_ORE * 4_545_992 => ONE_ORE,
        n if n < ONE_ORE * 4_616_993 => ONE_ORE,
        n if n < ONE_ORE * 4_680_894 => ONE_ORE,
        n if n < ONE_ORE * 4_738_404 => ONE_ORE,
        n if n < ONE_ORE * 4_790_164 => ONE_ORE,
        n if n < ONE_ORE * 4_836_748 => ONE_ORE,
        n if n < ONE_ORE * 4_878_673 => ONE_ORE,
        n if n < ONE_ORE * 4_916_406 => ONE_ORE,
        n if n < ONE_ORE * 4_950_365 => ONE_ORE,
        n if n < ONE_ORE * 4_980_928 => ONE_ORE,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Uniform, Rng};

    use crate::calculate_new_reward_rate;
    use ore_api::consts::{
        BASE_REWARD_RATE_MIN_THRESHOLD, BUS_COUNT, EPOCH_MINUTES, ONE_ORE, SMOOTHING_FACTOR,
    };

    const FUZZ_SIZE: u64 = 10_000;
    const TARGET_EPOCH_REWARDS: u64 = ONE_ORE * EPOCH_MINUTES as u64;
    const MAX_EPOCH_REWARDS: u64 = TARGET_EPOCH_REWARDS * BUS_COUNT as u64;

    #[test]
    fn test_calculate_new_reward_rate_target() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_div_by_zero() {
        let current_rate = 1000;
        let new_rate =
            calculate_new_reward_rate(current_rate, 0, TARGET_EPOCH_REWARDS, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_EPOCH_REWARDS.saturating_add(1_000_000_000),
            TARGET_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower_edge() {
        let current_rate = BASE_REWARD_RATE_MIN_THRESHOLD;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_EPOCH_REWARDS + 1,
            TARGET_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, TARGET_EPOCH_REWARDS));
            let actual_rewards: u64 =
                rng.sample(Uniform::new(TARGET_EPOCH_REWARDS, MAX_EPOCH_REWARDS));
            let new_rate = calculate_new_reward_rate(
                current_rate,
                actual_rewards,
                TARGET_EPOCH_REWARDS,
                TARGET_EPOCH_REWARDS,
            );
            assert!(new_rate.lt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_higher() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_EPOCH_REWARDS.saturating_sub(1_000_000_000),
            TARGET_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.gt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_higher_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, TARGET_EPOCH_REWARDS));
            let actual_rewards: u64 = rng.sample(Uniform::new(1, TARGET_EPOCH_REWARDS));
            let new_rate = calculate_new_reward_rate(
                current_rate,
                actual_rewards,
                TARGET_EPOCH_REWARDS,
                TARGET_EPOCH_REWARDS,
            );
            assert!(new_rate.gt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_max_smooth() {
        let current_rate = 1000;
        let new_rate =
            calculate_new_reward_rate(current_rate, 1, TARGET_EPOCH_REWARDS, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate.saturating_mul(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            u64::MAX,
            TARGET_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.eq(&current_rate.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_max_inputs() {
        let new_rate = calculate_new_reward_rate(
            TARGET_EPOCH_REWARDS,
            MAX_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.eq(&TARGET_EPOCH_REWARDS.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_inputs() {
        let new_rate = calculate_new_reward_rate(1, 1, TARGET_EPOCH_REWARDS, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&1u64.saturating_mul(SMOOTHING_FACTOR)));
    }
}
