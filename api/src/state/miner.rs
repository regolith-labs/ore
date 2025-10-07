use steel::*;

use crate::state::{miner_pda, Stake, Treasury};

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
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

    /// The amount of SOL this miner can claim.
    pub rewards_sol: u64,

    /// The amount of ORE this miner can claim.
    pub rewards_ore: u64,

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

    pub fn claim_ore(&mut self, amount: u64, stake: &mut Stake, treasury: &mut Treasury) -> u64 {
        stake.update_rewards(self, treasury);
        let amount = self.rewards_ore.min(amount);
        self.rewards_ore -= amount;
        treasury.total_unclaimed -= amount;

        // Charge a 10% fee and share with stakers.
        // if treasury.total_yielding_ore() > 0 {
        //     let claim_fee = amount / 10;
        //     amount -= claim_fee;
        //     treasury.update_rewards_factor(claim_fee);
        // }

        amount
    }
}

account!(OreAccount, Miner);
