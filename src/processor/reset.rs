use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    error::OreError,
    loaders::*,
    state::{Bus, Treasury},
    utils::AccountDeserialize,
    BUS_COUNT, BUS_EPOCH_REWARDS, EPOCH_DURATION, MAX_EPOCH_REWARDS, SMOOTHING_FACTOR,
    TARGET_EPOCH_REWARDS, TREASURY,
};

pub fn process_reset<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, mint_info, treasury_info, treasury_tokens_info, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_bus(bus_0_info, true)?;
    load_bus(bus_1_info, true)?;
    load_bus(bus_2_info, true)?;
    load_bus(bus_3_info, true)?;
    load_bus(bus_4_info, true)?;
    load_bus(bus_5_info, true)?;
    load_bus(bus_6_info, true)?;
    load_bus(bus_7_info, true)?;
    load_mint(mint_info, true)?;
    load_treasury(treasury_info, true)?;
    load_token_account(
        treasury_tokens_info,
        Some(treasury_info.key),
        mint_info.key,
        true,
    )?;
    load_sysvar(token_program, spl_token::id())?;
    let busses: [&AccountInfo; 8] = [
        bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info,
        bus_7_info,
    ];

    // Validate epoch has ended
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    let mut treasury_data = treasury_info.data.borrow_mut();
    let mut treasury = Treasury::try_from_bytes_mut(&mut treasury_data)?;
    let epoch_end_at = treasury.epoch_start_at.saturating_add(EPOCH_DURATION);
    if clock.unix_timestamp.lt(&epoch_end_at) {
        return Err(OreError::EpochActive.into());
    }

    // Reset busses
    let mut total_available_rewards = 0u64;
    for i in 0..BUS_COUNT {
        let mut bus_data = busses[i].data.borrow_mut();
        let mut bus = Bus::try_from_bytes_mut(&mut bus_data)?;
        total_available_rewards = total_available_rewards.saturating_add(bus.available_rewards);
        bus.available_rewards = BUS_EPOCH_REWARDS;
    }

    // Update the reward rate for the next epoch
    let total_epoch_rewards = MAX_EPOCH_REWARDS.saturating_sub(total_available_rewards);
    treasury.reward_rate = calculate_new_reward_rate(treasury.reward_rate, total_epoch_rewards);
    treasury.epoch_start_at = clock.unix_timestamp;

    // Top up treasury token account
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

    // TODO Logs?

    Ok(())
}

pub(crate) fn calculate_new_reward_rate(current_rate: u64, epoch_rewards: u64) -> u64 {
    // Avoid division by zero. Leave the reward rate unchanged.
    if epoch_rewards.eq(&0) {
        return current_rate;
    }

    // Calculate new reward rate.
    let new_rate = (current_rate as u128)
        .saturating_mul(TARGET_EPOCH_REWARDS as u128)
        .saturating_div(epoch_rewards as u128) as u64;

    // Smooth reward rate to not change by more than a constant factor from one epoch to the next.
    let new_rate_min = current_rate.saturating_div(SMOOTHING_FACTOR);
    let new_rate_max = current_rate.saturating_mul(SMOOTHING_FACTOR);
    let new_rate_smoothed = new_rate_min.max(new_rate_max.min(new_rate));

    // Prevent reward rate from dropping below 1 or exceeding BUS_EPOCH_REWARDS and return.
    new_rate_smoothed.max(1).min(BUS_EPOCH_REWARDS)
}

#[cfg(test)]
mod tests {
    use crate::{calculate_new_reward_rate, SMOOTHING_FACTOR, TARGET_EPOCH_REWARDS};

    #[test]
    fn test_calculate_new_reward_rate_stable() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_no_chage() {
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
    fn test_calculate_new_reward_rate_higher() {
        let current_rate = 1000;
        let new_rate =
            calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS.saturating_sub(1_000_000));
        assert!(new_rate.gt(&current_rate));
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
}
