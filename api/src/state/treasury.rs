use steel::*;

use super::OreAccount;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {
    // The amount of SOL collected for buy-bury operations.
    pub balance: u64,

    /// The amount of ORE in the motherlode rewards pool.
    pub motherlode: u64,

    /// The cumulative ORE distributed to stakers, divided by the total stake at the time of distribution.
    pub rewards_factor: Numeric,

    /// The current total amount of ORE staking deposits.
    pub total_staked: u64,

    /// The current total amount of unclaimed ORE mining rewards.
    pub total_unclaimed: u64,
}

impl Treasury {
    pub fn update_rewards_factor(&mut self, new_rewards: u64) {
        if self.total_staked == 0 {
            return;
        }
        self.rewards_factor += Numeric::from_fraction(new_rewards, self.total_staked);
    }
}

account!(OreAccount, Treasury);
