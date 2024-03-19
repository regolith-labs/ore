use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    error::OreError,
    loaders::*,
    state::{Bus, Treasury},
    utils::AccountDeserialize,
    BUS_COUNT, BUS_EPOCH_REWARDS, EPOCH_DURATION, MAX_EPOCH_REWARDS, SMOOTHING_FACTOR, START_AT,
    TARGET_EPOCH_REWARDS, TREASURY,
};

/// Reset transitions the Ore program from one epoch to the next. Its responsibilities include:
/// 1. Reset bus account rewards counters.
/// 2. Adjust the reward rate to stabilize inflation.
/// 3. Top up the treasury token account to backup claims.
///
/// Safety requirements:
/// - Reset is a permissionless crank function and can be invoked by anyone.
/// - Can only succeed if more 60 seconds or more have passed since the last successful reset.
/// - The busses, mint, treasury, treasury token account, and token program must all be valid.
///
/// Discussion:
/// - It is critical that `reset` can only be invoked once per 60 second period to ensure the supply growth rate
///   stays within the guaranteed bounds of 0 ≤ R ≤ 2 ORE/min.
/// - The reward rate is dynamically adjusted based on last epoch's actual reward rate (measured hashpower) to
///   target an average supply growth rate of 1 ORE/min.
pub fn process_reset<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, mint_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_bus(bus_0_info, 0, true)?;
    load_bus(bus_1_info, 1, true)?;
    load_bus(bus_2_info, 2, true)?;
    load_bus(bus_3_info, 3, true)?;
    load_bus(bus_4_info, 4, true)?;
    load_bus(bus_5_info, 5, true)?;
    load_bus(bus_6_info, 6, true)?;
    load_bus(bus_7_info, 7, true)?;
    load_mint(mint_info, true)?;
    load_treasury(treasury_info, true)?;
    load_token_account(
        treasury_tokens_info,
        Some(treasury_info.key),
        mint_info.key,
        true,
    )?;
    load_program(token_program, spl_token::id())?;
    let busses: [&AccountInfo; BUS_COUNT] = [
        bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info,
        bus_7_info,
    ];

    // Validate mining has starting
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    if clock.unix_timestamp.lt(&START_AT) {
        return Err(OreError::NotStarted.into());
    }

    // Validate at least 60 seconds have passed since last reset
    let mut treasury_data = treasury_info.data.borrow_mut();
    let treasury = Treasury::try_from_bytes_mut(&mut treasury_data)?;
    let threshold = treasury.last_reset_at.saturating_add(EPOCH_DURATION);
    if clock.unix_timestamp.lt(&threshold) {
        return Err(OreError::ResetTooEarly.into());
    }

    // Record current timestamp
    treasury.last_reset_at = clock.unix_timestamp;

    // Reset bus accounts and calculate actual rewards mined since last reset
    let mut total_remaining_rewards = 0u64;
    for i in 0..BUS_COUNT {
        let mut bus_data = busses[i].data.borrow_mut();
        let bus = Bus::try_from_bytes_mut(&mut bus_data)?;
        total_remaining_rewards = total_remaining_rewards.saturating_add(bus.rewards);
        bus.rewards = BUS_EPOCH_REWARDS;
    }
    let total_epoch_rewards = MAX_EPOCH_REWARDS.saturating_sub(total_remaining_rewards);

    // Update reward rate for next epoch
    treasury.reward_rate = calculate_new_reward_rate(treasury.reward_rate, total_epoch_rewards);

    // Fund treasury token account
    let treasury_bump = treasury.bump as u8;
    drop(treasury_data);
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
        &[&[TREASURY, &[treasury_bump]]],
    )?;

    Ok(())
}

/// This function calculates what the new reward rate should be based on how many total rewards were mined in the prior epoch.
/// The math is largely identitical to that used by the Bitcoin network for updating the difficulty between each epoch.
/// new_rate = current_rate * (target_rewards / actual_rewards)
/// The new rate is then smoothed by a constant factor to avoid unexpectedly large fluctuations.
/// In Ore's case, the epochs are so short (60 seconds) that the smoothing factor of 2 has been chosen.
pub(crate) fn calculate_new_reward_rate(current_rate: u64, epoch_rewards: u64) -> u64 {
    // Avoid division by zero. Leave the reward rate unchanged, if detected.
    if epoch_rewards.eq(&0) {
        return current_rate;
    }

    // Calculate new reward rate.
    let new_rate = (current_rate)
        .saturating_mul(TARGET_EPOCH_REWARDS)
        .saturating_div(epoch_rewards) as u64;

    // Smooth reward rate so it cannot change by more than a constant factor from one epoch to the next.
    let new_rate_min = current_rate.saturating_div(SMOOTHING_FACTOR);
    let new_rate_max = current_rate.saturating_mul(SMOOTHING_FACTOR);
    let new_rate_smoothed = new_rate_min.max(new_rate_max.min(new_rate));

    // Prevent reward rate from dropping below 1 or exceeding BUS_EPOCH_REWARDS and return.
    new_rate_smoothed.max(1).min(BUS_EPOCH_REWARDS)
}

#[cfg(test)]
mod tests {
    use rand::{distributions::Uniform, Rng};

    use crate::{
        calculate_new_reward_rate, BUS_EPOCH_REWARDS, MAX_EPOCH_REWARDS, SMOOTHING_FACTOR,
        TARGET_EPOCH_REWARDS,
    };

    const FUZZ_SIZE: u64 = 10_000;

    #[test]
    fn test_calculate_new_reward_rate_target() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_div_by_zero() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 0);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower() {
        let current_rate = 1000;
        let new_rate =
            calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS.saturating_add(1_000_000));
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, BUS_EPOCH_REWARDS));
            let actual_rewards: u64 =
                rng.sample(Uniform::new(TARGET_EPOCH_REWARDS, MAX_EPOCH_REWARDS));
            let new_rate = calculate_new_reward_rate(current_rate, actual_rewards);
            assert!(new_rate.lt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_higher() {
        let current_rate = 1000;
        let new_rate =
            calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS.saturating_sub(1_000_000));
        println!("{:?} {:?}", new_rate, current_rate);
        assert!(new_rate.gt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_higher_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, BUS_EPOCH_REWARDS));
            let actual_rewards: u64 = rng.sample(Uniform::new(1, TARGET_EPOCH_REWARDS));
            let new_rate = calculate_new_reward_rate(current_rate, actual_rewards);
            assert!(new_rate.gt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_max_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 1);
        assert!(new_rate.eq(&current_rate.saturating_mul(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, u64::MAX);
        assert!(new_rate.eq(&current_rate.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_max_inputs() {
        let new_rate = calculate_new_reward_rate(BUS_EPOCH_REWARDS, MAX_EPOCH_REWARDS);
        assert!(new_rate.eq(&BUS_EPOCH_REWARDS.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_inputs() {
        let new_rate = calculate_new_reward_rate(1, 1);
        assert!(new_rate.eq(&1u64.saturating_mul(SMOOTHING_FACTOR)));
    }
}
