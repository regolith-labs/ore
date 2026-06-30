use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{miner_pda, OreAccount, Treasury};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Miner {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// Whether or not to auto-return SOL winnings to the miner's wallet.
    pub auto_return: u64,

    /// The checkpoint ID.
    pub checkpoint_id: u64,

    /// SOL witheld in reserve to pay for checkpointing.
    pub checkpoint_fee: u64,

    /// The amount of SOL deployed on each square.
    /// TODO: Rename to sol.
    pub deployed: [u64; 25],

    /// The amount of SOL deployed on each square, weighted by the time remaining when deployed.
    pub mass: [u64; 25],

    /// The cumulative mass on each square prior to this miner's deployment on that square.
    /// TODO: Rename to mass_cumulative.
    pub cumulative: [u64; 25],

    /// The round ID.
    pub round_id: u64,

    /// The rewards factor last time rewards were updated on this miner account.
    pub rewards_factor: Numeric,

    /// The amount of SOL this miner has had returned and may claim.
    /// TODO: Rename to sol_returned.
    pub rewards_sol: u64,

    /// The amount of ORE this miner has earned from refining fees and may claim.
    /// TODO: Rename to ore_refined.
    pub refined_ore: u64,

    /// The amount of ORE this miner has mined and may claim.
    /// TODO: Rename to ore_unrefined.
    pub rewards_ore: u64,

    /// The last time this miner claimed ORE rewards.
    pub last_claim_ore_at: i64,

    /// The last time this miner claimed SOL rewards.
    pub last_claim_sol_at: i64,

    /// The total amount of ORE this miner has mined across all blocks.
    /// TODO: Rename to lifetime_rewards_ore.
    pub lifetime_rewards_ore: u64,

    /// The total amount of SOL this miner has deployed across all rounds.
    pub lifetime_deployed: u64,

    /// The total amount of SOL this miner has mined across all blocks.
    /// TODO: Rename to lifetime_returned_sol.
    pub lifetime_rewards_sol: u64,
}

impl Miner {
    pub fn pda(&self) -> (Pubkey, u8) {
        miner_pda(self.authority)
    }

    pub fn claim_ore(&mut self, amount: u64, clock: &Clock, treasury: &mut Treasury) -> u64 {
        self.update_rewards(treasury);

        let refined_available = self.refined_ore;
        let rewards_available = self.rewards_ore;
        let total_available = refined_available + rewards_available;
        let amount_to_claim = amount.min(total_available);

        // Distribute the claim evenly between refined_ore and rewards_ore, but if one is low, take the remainder from the other.
        let mut refined_to_claim = (amount_to_claim / 2).min(refined_available);
        let mut rewards_to_claim = (amount_to_claim / 2).min(rewards_available);

        // If uneven division or one source is low, allocate additional from the other
        let claimed_so_far = refined_to_claim + rewards_to_claim;
        if claimed_so_far < amount_to_claim {
            // deduct remainder from whichever has more left
            let remainder = amount_to_claim - claimed_so_far;
            if refined_available - refined_to_claim >= rewards_available - rewards_to_claim {
                let extra = remainder.min(refined_available - refined_to_claim);
                refined_to_claim += extra;
                // if still not enough, add to rewards_to_claim
                if refined_to_claim + rewards_to_claim < amount_to_claim {
                    rewards_to_claim += amount_to_claim - (refined_to_claim + rewards_to_claim);
                }
            } else {
                let extra = remainder.min(rewards_available - rewards_to_claim);
                rewards_to_claim += extra;
                // if still not enough, add to refined_to_claim
                if refined_to_claim + rewards_to_claim < amount_to_claim {
                    refined_to_claim += amount_to_claim - (refined_to_claim + rewards_to_claim);
                }
            }
        }

        // Remove claimed amounts from miner's balances and treasury totals
        self.refined_ore -= refined_to_claim;
        self.rewards_ore -= rewards_to_claim;
        treasury.total_refined = treasury.total_refined.saturating_sub(refined_to_claim);
        treasury.total_unclaimed = treasury.total_unclaimed.saturating_sub(rewards_to_claim);
        self.last_claim_ore_at = clock.unix_timestamp;

        // Rewards portion pays a 10% fee; refined pays 0%.
        if treasury.total_unclaimed > 0 && rewards_to_claim > 0 {
            let fee = rewards_to_claim / 10;
            // Only deduct fee up to actual rewards claimed (prevents underflow).
            let claim_after_fee = (refined_to_claim) + (rewards_to_claim - fee);
            treasury.miner_rewards_factor += Numeric::from_fraction(fee, treasury.total_unclaimed);
            treasury.total_refined += fee;
            self.lifetime_rewards_ore = self.lifetime_rewards_ore.saturating_sub(fee);
            return claim_after_fee;
        }

        // If no fee (all from refined, or no unclaimed to share), return total claimed.
        refined_to_claim + rewards_to_claim
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
