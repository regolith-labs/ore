use serde::{Deserialize, Serialize};

use crate::state::{miner_box_name, Numeric, Treasury};

/// Miner account state
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Miner {
    /// The authority of this miner account (Algorand address bytes).
    pub authority: [u8; 32],

    /// The miner's prospects in the current round.
    pub deployed: [u64; 25],

    /// The cumulative amount of ALGO deployed on each square prior to this miner's move.
    pub cumulative: [u64; 25],

    /// ALGO withheld in reserve to pay for checkpointing (in microalgos).
    pub checkpoint_fee: u64,

    /// The last round that this miner checkpointed.
    pub checkpoint_id: u64,

    /// The last time this miner claimed fPOW rewards (unix timestamp).
    pub last_claim_fpow_at: i64,

    /// The last time this miner claimed ALGO rewards (unix timestamp).
    pub last_claim_algo_at: i64,

    /// The rewards factor last time rewards were updated on this miner account.
    pub rewards_factor: Numeric,

    /// The amount of ALGO this miner can claim (in microalgos).
    pub rewards_algo: u64,

    /// The amount of fPOW this miner can claim.
    pub rewards_fpow: u64,

    /// The amount of fPOW this miner has earned from claim fees.
    pub refined_fpow: u64,

    /// The ID of the round this miner last played in.
    pub round_id: u64,

    /// The total amount of ALGO this miner has mined across all blocks.
    pub lifetime_rewards_algo: u64,

    /// The total amount of fPOW this miner has mined across all blocks.
    pub lifetime_rewards_fpow: u64,

    /// The total amount of ALGO this miner has deployed across all rounds.
    pub lifetime_deployed: u64,
}

impl Miner {
    pub fn box_name(&self) -> Vec<u8> {
        miner_box_name(&self.authority)
    }

    pub fn claim_fpow(&mut self, timestamp: i64, treasury: &mut Treasury) -> u64 {
        self.update_rewards(treasury);
        let refined_fpow = self.refined_fpow;
        let rewards_fpow = self.rewards_fpow;
        let mut amount = refined_fpow + rewards_fpow;
        self.refined_fpow = 0;
        self.rewards_fpow = 0;
        treasury.total_unclaimed -= rewards_fpow;
        treasury.total_refined -= refined_fpow;
        self.last_claim_fpow_at = timestamp;

        // Charge a 10% fee and share with miners who haven't claimed yet.
        if treasury.total_unclaimed > 0 {
            let fee = rewards_fpow / 10;
            amount -= fee;
            treasury.miner_rewards_factor =
                treasury.miner_rewards_factor + Numeric::from_fraction(fee, treasury.total_unclaimed);
            treasury.total_refined += fee;
            self.lifetime_rewards_fpow -= fee;
        }

        amount
    }

    pub fn claim_algo(&mut self, timestamp: i64) -> u64 {
        let amount = self.rewards_algo;
        self.rewards_algo = 0;
        self.last_claim_algo_at = timestamp;
        amount
    }

    pub fn update_rewards(&mut self, treasury: &Treasury) {
        // Accumulate rewards, weighted by stake balance.
        if treasury.miner_rewards_factor > self.rewards_factor {
            let accumulated_rewards = treasury.miner_rewards_factor - self.rewards_factor;
            if accumulated_rewards < Numeric::ZERO {
                panic!("Accumulated rewards is negative");
            }
            let personal_rewards = accumulated_rewards * Numeric::from_u64(self.rewards_fpow);
            self.refined_fpow += personal_rewards.to_u64();
            self.lifetime_rewards_fpow += personal_rewards.to_u64();
        }

        // Update this miner account's last seen rewards factor.
        self.rewards_factor = treasury.miner_rewards_factor;
    }
}

impl Default for Miner {
    fn default() -> Self {
        Self {
            authority: [0u8; 32],
            deployed: [0u64; 25],
            cumulative: [0u64; 25],
            checkpoint_fee: 0,
            checkpoint_id: 0,
            last_claim_fpow_at: 0,
            last_claim_algo_at: 0,
            rewards_factor: Numeric::ZERO,
            rewards_algo: 0,
            rewards_fpow: 0,
            refined_fpow: 0,
            round_id: 0,
            lifetime_rewards_algo: 0,
            lifetime_rewards_fpow: 0,
            lifetime_deployed: 0,
        }
    }
}
