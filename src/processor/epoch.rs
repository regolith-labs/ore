use solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, pubkey::Pubkey};

use crate::{BUS_EPOCH_REWARDS, SMOOTHING_FACTOR, TARGET_EPOCH_REWARDS};

pub fn process_epoch<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // TODO
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
