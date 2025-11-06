use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{stake_pda, Treasury};

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Stake {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The balance of this stake account.
    pub balance: u64,

    /// The timestamp of last claim.
    pub last_claim_at: i64,

    /// The timestamp the last time this staker deposited.
    pub last_deposit_at: i64,

    /// The timestamp the last time this staker withdrew.
    pub last_withdraw_at: i64,

    /// The rewards factor last time rewards were updated on this stake account.
    pub rewards_factor: Numeric,

    /// The amount of ORE this staker can claim.
    pub rewards: u64,

    /// The total amount of ORE this staker has earned over its lifetime.
    pub lifetime_rewards: u64,

    /// Flag indicating whether this staker is associated with a Solana Seeker.
    pub is_seeker: u64,
}

impl Stake {
    pub fn pda(&self) -> (Pubkey, u8) {
        stake_pda(self.authority)
    }

    pub fn claim(&mut self, amount: u64, clock: &Clock, treasury: &Treasury) -> u64 {
        self.update_rewards(treasury);
        let amount = self.rewards.min(amount);
        self.rewards -= amount;
        self.last_claim_at = clock.unix_timestamp;
        amount
    }

    pub fn deposit(
        &mut self,
        amount: u64,
        clock: &Clock,
        treasury: &mut Treasury,
        sender: &TokenAccount,
    ) -> u64 {
        self.update_rewards(treasury);
        let amount = sender.amount().min(amount);
        self.balance += amount;
        self.last_deposit_at = clock.unix_timestamp;
        treasury.total_staked += amount;
        amount
    }

    pub fn withdraw(&mut self, amount: u64, clock: &Clock, treasury: &mut Treasury) -> u64 {
        self.update_rewards(treasury);
        let amount = self.balance.min(amount);
        self.balance -= amount;
        self.last_withdraw_at = clock.unix_timestamp;
        treasury.total_staked -= amount;
        amount
    }

    pub fn update_rewards(&mut self, treasury: &Treasury) {
        // Accumulate rewards, weighted by stake balance.
        if treasury.stake_rewards_factor > self.rewards_factor {
            let accumulated_rewards = treasury.stake_rewards_factor - self.rewards_factor;
            if accumulated_rewards < Numeric::ZERO {
                panic!("Accumulated rewards is negative");
            }
            let personal_rewards = accumulated_rewards * Numeric::from_u64(self.balance);
            self.rewards += personal_rewards.to_u64();
            self.lifetime_rewards += personal_rewards.to_u64();
        }

        // Update this stake account's last seen rewards factor.
        self.rewards_factor = treasury.stake_rewards_factor;
    }
}

account!(OreAccount, Stake);
