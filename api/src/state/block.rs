use steel::*;

use crate::state::block_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Block {
    /// The block number.
    pub id: u64,

    /// The minimum difficulty required for payout.
    pub min_difficulty: u64,

    /// The reward rate per satisfying hash.
    pub reward_rate: u64,

    /// The hash of the starting slot.
    pub slot_hash: [u8; 32],

    /// The starting slot of the block.
    pub start_slot: u64,

    /// The total number of hashes submitted to the block.
    pub total_hashes: u64,

    /// The total amount of rewards paid out to miners.
    pub total_rewards: u64,

    /// The total number of hashes submitted to the block.
    pub winning_hashes: u64,
}

impl Block {
    pub fn pda(&self) -> (Pubkey, u8) {
        block_pda(self.id)
    }
}

account!(OreAccount, Block);
