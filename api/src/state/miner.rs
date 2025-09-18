use steel::*;

use crate::state::{miner_pda, OreAccountOLD};

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Miner {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The miner's prospects in the current round.
    pub prospects: [u64; 25],

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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MinerOLD {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The ID of the last block this miner mined in.
    pub block_id: u64,

    /// An account authorized to execute actions on behalf of this miner.
    pub executor: Pubkey,

    /// The amount of hashpower this miner has committed to the current block.
    pub hashpower: u64,

    /// A user-supplied seed for random number generation.
    pub seed: [u8; 32],

    /// The total amount of hashpower this miner has committed across all blocks.
    pub total_hashpower: u64,

    /// The total amount of ORE this miner has mined across all blocks.
    pub total_rewards: u64,
}

impl Miner {
    pub fn pda(&self) -> (Pubkey, u8) {
        miner_pda(self.authority)
    }
}

account!(OreAccount, Miner);
account!(OreAccountOLD, MinerOLD);
