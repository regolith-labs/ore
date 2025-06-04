use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Block {
    /// The best hash.
    pub best_hash: [u8; 32],

    /// The miner who submitted the best hash.
    pub best_miner: Pubkey,

    /// The block number.
    pub id: u64,

    /// The amount of ORE to payout to the miner who submitted the best hash.
    pub reward: u64,

    /// The hash of the starting slot.
    pub slot_hash: [u8; 32],

    /// The starting slot of the block.
    pub start_slot: u64,
}

account!(OreAccount, Block);
