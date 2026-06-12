use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{miner_pda, OreAccountV4, Treasury};

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

    /// The total amount of SOL this miner has deployed across all rounds.
    pub lifetime_deployed: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct MinerV4 {
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

    /// The cumulative sol on each square prior to this miner's deployment on that square.
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

    /// The total amount of SOL this miner has returned across all blocks.
    pub lifetime_returned_sol: u64,

    /// The total amount of ORE this miner has mined across all blocks.
    pub lifetime_rewards: u64,

    /// The total amount of SOL this miner has deployed across all rounds.
    pub lifetime_deployed: u64,
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

impl MinerV4 {
    pub fn pda(&self) -> (Pubkey, u8) {
        miner_pda(self.authority)
    }

    pub fn claim_ore(&mut self, clock: &Clock, treasury: &mut Treasury) -> u64 {
        self.update_rewards(treasury);
        let refined_ore = self.ore_refined;
        let rewards_ore = self.ore_unrefined;
        let mut amount = refined_ore + rewards_ore;
        self.ore_refined = 0;
        self.ore_unrefined = 0;
        treasury.total_unclaimed -= rewards_ore;
        treasury.total_refined -= refined_ore;
        self.last_claim_ore_at = clock.unix_timestamp;

        // Charge a 10% fee and share with miners who haven't claimed yet.
        if treasury.total_unclaimed > 0 {
            let fee = rewards_ore / 10;
            amount -= fee;
            treasury.miner_rewards_factor += Numeric::from_fraction(fee, treasury.total_unclaimed);
            treasury.total_refined += fee;
            self.lifetime_rewards -= fee;
        }

        amount
    }

    pub fn claim_sol(&mut self, clock: &Clock) -> u64 {
        let amount = self.sol_returned;
        self.sol_returned = 0;
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
            let personal_rewards = accumulated_rewards * Numeric::from_u64(self.ore_unrefined);
            self.ore_refined += personal_rewards.to_u64();
            self.lifetime_rewards += personal_rewards.to_u64();
        }

        // Update this miner account's last seen rewards factor.
        self.rewards_factor = treasury.miner_rewards_factor;
    }
}

account!(OreAccount, Miner);
account!(OreAccountV4, MinerV4);

pub enum MinerAccount {
    Miner(Miner),
    MinerV4(MinerV4),
}

impl MinerAccount {
    pub fn authority(&self) -> Pubkey {
        match self {
            MinerAccount::Miner(miner) => miner.authority,
            MinerAccount::MinerV4(miner) => miner.authority,
        }
    }

    pub fn deployed(&self) -> [u64; 25] {
        match self {
            MinerAccount::Miner(miner) => miner.deployed,
            MinerAccount::MinerV4(miner) => miner.sol,
        }
    }

    pub fn cumulative(&self) -> [u64; 25] {
        match self {
            MinerAccount::Miner(miner) => miner.cumulative,
            MinerAccount::MinerV4(miner) => miner.sol_cumulative,
        }
    }

    pub fn checkpoint_fee(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.checkpoint_fee,
            MinerAccount::MinerV4(miner) => miner.checkpoint_fee,
        }
    }

    pub fn checkpoint_id(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.checkpoint_id,
            MinerAccount::MinerV4(miner) => miner.checkpoint_id,
        }
    }

    pub fn last_claim_ore_at(&self) -> i64 {
        match self {
            MinerAccount::Miner(miner) => miner.last_claim_ore_at,
            MinerAccount::MinerV4(miner) => miner.last_claim_ore_at,
        }
    }

    pub fn last_claim_sol_at(&self) -> i64 {
        match self {
            MinerAccount::Miner(miner) => miner.last_claim_sol_at,
            MinerAccount::MinerV4(miner) => miner.last_claim_sol_at,
        }
    }
    pub fn rewards_factor(&self) -> Numeric {
        match self {
            MinerAccount::Miner(miner) => miner.rewards_factor,
            MinerAccount::MinerV4(miner) => miner.rewards_factor,
        }
    }

    pub fn rewards_sol(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.rewards_sol,
            MinerAccount::MinerV4(miner) => miner.sol_returned,
        }
    }

    pub fn rewards_ore(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.rewards_ore,
            MinerAccount::MinerV4(miner) => miner.ore_unrefined,
        }
    }

    pub fn refined_ore(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.refined_ore,
            MinerAccount::MinerV4(miner) => miner.ore_refined,
        }
    }

    pub fn round_id(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.round_id,
            MinerAccount::MinerV4(miner) => miner.round_id,
        }
    }

    pub fn lifetime_rewards_sol(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.lifetime_rewards_sol,
            MinerAccount::MinerV4(miner) => miner.lifetime_returned_sol,
        }
    }

    pub fn lifetime_rewards_ore(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.lifetime_rewards_ore,
            MinerAccount::MinerV4(miner) => miner.lifetime_rewards,
        }
    }

    pub fn lifetime_deployed(&self) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.lifetime_deployed,
            MinerAccount::MinerV4(miner) => miner.lifetime_deployed,
        }
    }

    pub fn pda(&self) -> (Pubkey, u8) {
        match self {
            MinerAccount::Miner(miner) => miner.pda(),
            MinerAccount::MinerV4(miner) => miner.pda(),
        }
    }

    pub fn claim_ore(&mut self, clock: &Clock, treasury: &mut Treasury) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.claim_ore(clock, treasury),
            MinerAccount::MinerV4(miner) => miner.claim_ore(clock, treasury),
        }
    }

    pub fn claim_sol(&mut self, clock: &Clock) -> u64 {
        match self {
            MinerAccount::Miner(miner) => miner.claim_sol(clock),
            MinerAccount::MinerV4(miner) => miner.claim_sol(clock),
        }
    }

    pub fn update_rewards(&mut self, treasury: &Treasury) {
        match self {
            MinerAccount::Miner(miner) => miner.update_rewards(treasury),
            MinerAccount::MinerV4(miner) => miner.update_rewards(treasury),
        }
    }
}
