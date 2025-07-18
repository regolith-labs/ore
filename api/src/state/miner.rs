use steel::*;

use crate::state::miner_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Miner {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The ID of the last block this miner mined in.
    pub block_id: u64,

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
