use coal_api::{consts::*, state::{Config, WoodConfig}};
use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult,
    program_error::ProgramError,
};

use crate::utils::Discriminator;

use reset_coal::*;
use reset_wood::*;

mod reset_coal;
mod reset_wood;

pub fn process_reset<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let config_info = &accounts[9];

    if config_info.data.borrow()[0].eq(&(Config::discriminator() as u8)) {
        return process_reset_coal(accounts, data)
    }

    if config_info.data.borrow()[0].eq(&(WoodConfig::discriminator() as u8)) {
        return process_reset_wood(accounts, data)
    }

    return Err(ProgramError::InvalidAccountData);    
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
pub(crate) fn calculate_new_reward_rate(current_rate: u64, epoch_rewards: u64, target_rewards: u64, bus_rewards: u64) -> u64 {
    // Avoid division by zero. Leave the reward rate unchanged, if detected.
    if epoch_rewards.eq(&0) {
        return current_rate;
    }

    // Calculate new reward rate.
    let new_rate = (current_rate as u128)
        .saturating_mul(target_rewards as u128)
        .saturating_div(epoch_rewards as u128) as u64;

    // Smooth reward rate so it cannot change by more than a constant factor from one epoch to the next.
    let new_rate_min = current_rate.saturating_div(SMOOTHING_FACTOR);
    let new_rate_max = current_rate.saturating_mul(SMOOTHING_FACTOR);
    let new_rate_smoothed = new_rate.min(new_rate_max).max(new_rate_min);
    // Prevent reward rate from dropping below 1 or exceeding target_rewards and return.
    new_rate_smoothed.max(1).min(bus_rewards)
}