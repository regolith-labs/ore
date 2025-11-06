use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{miner_pda, Treasury};

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Miner {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The miner's prospects in the current round.
    pub deployed: [u64; 25],

    /// The cumulative amount of SOL deployed on each square prior to this miner's move.
    pub cumulative: [u64; 25],

    /// SOL witheld in reserve to pay for checkpointing.
    pub checkpoint_fee: u64,

    /// The last round that this miner checkpointed.
    pub checkpoint_id: u64,

    /// The last time this miner claimed ORE rewards.
    pub last_claim_ore_at: i64,

    /// The last time this miner claimed SOL rewards.
    pub last_claim_sol_at: i64,

    /// The rewards factor last time rewards were updated on this miner account.
    pub rewards_factor: Numeric,

    /// The amount of SOL this miner can claim.
    pub rewards_sol: u64,

    /// The amount of ORE this miner can claim.
    pub rewards_ore: u64,

    /// The amount of ORE this miner has earned from claim fees.
    pub refined_ore: u64,

    /// The ID of the round this miner last played in.
    pub round_id: u64,

    /// The total amount of SOL this miner has mined across all blocks.
    pub lifetime_rewards_sol: u64,

    /// The total amount of ORE this miner has mined across all blocks.
    pub lifetime_rewards_ore: u64,
}

impl Miner {
    pub fn pda(&self) -> (Pubkey, u8) {
        miner_pda(self.authority)
    }

    pub fn claim_ore(&mut self, clock: &Clock, treasury: &mut Treasury) -> u64 {
        self.update_rewards(treasury);
        let refined_ore = self.refined_ore;
        let rewards_ore = self.rewards_ore;
        let mut amount = refined_ore + rewards_ore;
        self.refined_ore = 0;
        self.rewards_ore = 0;
        treasury.total_unclaimed -= rewards_ore;
        treasury.total_refined -= refined_ore;
        self.last_claim_ore_at = clock.unix_timestamp;

        // Charge a 10% fee and share with miners who haven't claimed yet.
        if treasury.total_unclaimed > 0 {
            let fee = rewards_ore / 10;
            amount -= fee;
            treasury.miner_rewards_factor += Numeric::from_fraction(fee, treasury.total_unclaimed);
            treasury.total_refined += fee;
            self.lifetime_rewards_ore -= fee;
        }

        amount
    }

    pub fn claim_sol(&mut self, clock: &Clock) -> u64 {
        let amount = self.rewards_sol;
        self.rewards_sol = 0;
        self.last_claim_sol_at = clock.unix_timestamp;
        amount
    }

    pub fn update_rewards(&mut self, treasury: &Treasury) {
        // Accumulate rewards, weighted by stake balance.
        if treasury.miner_rewards_factor > self.rewards_factor {
            let accumulated_rewards = treasury.miner_rewards_factor - self.rewards_factor;
            if accumulated_rewards < Numeric::ZERO {
                panic!("Accumulated rewards is negative");
            }
            let personal_rewards = accumulated_rewards * Numeric::from_u64(self.rewards_ore);
            self.refined_ore += personal_rewards.to_u64();
            self.lifetime_rewards_ore += personal_rewards.to_u64();
        }

        // Update this miner account's last seen rewards factor.
        self.rewards_factor = treasury.miner_rewards_factor;
    }
}

account!(OreAccount, Miner);
