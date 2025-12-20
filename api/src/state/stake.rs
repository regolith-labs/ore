use serde::{Deserialize, Serialize};

use crate::state::{stake_box_name, Numeric, Treasury};

/// Stake account state for staking fPOW tokens
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Stake {
    /// The authority of this stake account (Algorand address bytes).
    pub authority: [u8; 32],

    /// The balance of this stake account.
    pub balance: u64,

    /// Buffer a (placeholder)
    pub buffer_a: u64,

    /// Buffer b (placeholder)
    pub buffer_b: u64,

    /// Buffer c (placeholder)
    pub buffer_c: u64,

    /// Buffer d (placeholder)
    pub buffer_d: u64,

    /// The microalgo reserve to pay fees for auto-compounding bots.
    pub compound_fee_reserve: u64,

    /// The timestamp of last claim (unix timestamp).
    pub last_claim_at: i64,

    /// The timestamp the last time this staker deposited (unix timestamp).
    pub last_deposit_at: i64,

    /// The timestamp the last time this staker withdrew (unix timestamp).
    pub last_withdraw_at: i64,

    /// The rewards factor last time rewards were updated on this stake account.
    pub rewards_factor: Numeric,

    /// The amount of fPOW this staker can claim.
    pub rewards: u64,

    /// The total amount of fPOW this staker has earned over its lifetime.
    pub lifetime_rewards: u64,

    /// Buffer f (placeholder)
    pub buffer_f: u64,
}

impl Stake {
    pub fn box_name(&self) -> Vec<u8> {
        stake_box_name(&self.authority)
    }

    pub fn claim(&mut self, amount: u64, timestamp: i64, treasury: &Treasury) -> u64 {
        self.update_rewards(treasury);
        let amount = self.rewards.min(amount);
        self.rewards -= amount;
        self.last_claim_at = timestamp;
        amount
    }

    pub fn deposit(
        &mut self,
        amount: u64,
        timestamp: i64,
        treasury: &mut Treasury,
        sender_balance: u64,
    ) -> u64 {
        self.update_rewards(treasury);
        let amount = sender_balance.min(amount);
        self.balance += amount;
        self.last_deposit_at = timestamp;
        treasury.total_staked += amount;
        amount
    }

    pub fn withdraw(&mut self, amount: u64, timestamp: i64, treasury: &mut Treasury) -> u64 {
        self.update_rewards(treasury);
        let amount = self.balance.min(amount);
        self.balance -= amount;
        self.last_withdraw_at = timestamp;
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

impl Default for Stake {
    fn default() -> Self {
        Self {
            authority: [0u8; 32],
            balance: 0,
            buffer_a: 0,
            buffer_b: 0,
            buffer_c: 0,
            buffer_d: 0,
            compound_fee_reserve: 0,
            last_claim_at: 0,
            last_deposit_at: 0,
            last_withdraw_at: 0,
            rewards_factor: Numeric::ZERO,
            rewards: 0,
            lifetime_rewards: 0,
            buffer_f: 0,
        }
    }
}
