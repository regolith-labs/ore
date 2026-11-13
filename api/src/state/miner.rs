use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{miner_pda, OreAccount, Treasury, DENOMINATOR_BPS};

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
    pub sol: [u64; 25],

    /// The amount of SOL deployed on each square, weighted by the time remaining when deployed.
    pub mass: [u64; 25],

    /// The cumulative SOL deployed on each square prior to this miner's deployment on that square.
    pub sol_cumulative: [u64; 25],

    /// The round ID.
    pub round_id: u64,

    /// The rewards factor last time rewards were updated on this miner account.
    pub rewards_factor: Numeric,

    /// The amount of SOL this miner has had returned and may claim.
    pub sol_returned: u64,

    /// The amount of ORE this miner has earned from refining fees and may claim.
    pub ore_refined: u64,

    /// The amount of ORE this miner has mined and may claim.
    pub ore_unrefined: u64,

    /// The last time this miner claimed ORE rewards.
    pub last_claim_ore_at: i64,

    /// The last time this miner claimed SOL rewards.
    pub last_claim_sol_at: i64,

    /// The total amount of ORE this miner has mined across all blocks.
    pub lifetime_rewards_ore: u64,

    /// The total amount of SOL this miner has deployed across all rounds.
    pub lifetime_deployed: u64,

    /// The total amount of SOL this miner has mined across all blocks.
    pub lifetime_returned_sol: u64,
}

impl Miner {
    pub fn pda(&self) -> (Pubkey, u8) {
        miner_pda(self.authority)
    }

    pub fn claim_ore(&mut self, clock: &Clock, treasury: &mut Treasury, bps: u64) -> (u64, u64) {
        self.update_rewards(treasury);

        // Compute % claimable
        // bps = basis points, so 10000 == 100%
        let bps = bps.min(DENOMINATOR_BPS);
        let claim_refined = (self.ore_refined * bps) / DENOMINATOR_BPS;
        let claim_rewards = (self.ore_unrefined * bps) / DENOMINATOR_BPS;

        // Withdraw amounts
        self.ore_refined -= claim_refined;
        self.ore_unrefined -= claim_rewards;
        treasury.total_refined -= claim_refined;
        treasury.total_unrefined -= claim_rewards;
        self.last_claim_ore_at = clock.unix_timestamp;

        // Apply 10% fee on unrefined portion (ore_unrefined)
        let mut fee = 0;
        let mut amount = claim_refined + claim_rewards;
        if claim_rewards > 0 && treasury.total_unrefined > 0 {
            fee = 1.max(claim_rewards / 10);
            amount -= fee;
            // Distribute the tax
            treasury.rewards_factor += Numeric::from_fraction(fee, treasury.total_unrefined);
            treasury.total_refined += fee;
            self.lifetime_rewards_ore -= fee;
        }

        (amount, fee)
    }

    pub fn claim_sol(&mut self, clock: &Clock) -> u64 {
        let amount = self.sol_returned;
        self.sol_returned = 0;
        self.last_claim_sol_at = clock.unix_timestamp;
        amount
    }

    pub fn update_rewards(&mut self, treasury: &Treasury) {
        // Accumulate rewards, weighted by stake balance.
        if treasury.rewards_factor > self.rewards_factor {
            let accumulated_rewards = treasury.rewards_factor - self.rewards_factor;
            if accumulated_rewards < Numeric::ZERO {
                panic!("Accumulated rewards is negative");
            }
            let personal_rewards = accumulated_rewards * Numeric::from_u64(self.ore_unrefined);
            self.ore_refined += personal_rewards.to_u64();
            self.lifetime_rewards_ore += personal_rewards.to_u64();
        }

        // Update this miner account's last seen rewards factor.
        self.rewards_factor = treasury.rewards_factor;
    }
}

account!(OreAccount, Miner);
