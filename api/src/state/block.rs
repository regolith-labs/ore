use steel::*;

use crate::state::block_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Block {
    /// The block number.
    pub id: u64,

    /// The party that opened the block.
    pub opener: Pubkey,

    /// The reward configuration.
    pub reward: u64,

    /// The best hash submitted to the block.
    pub best_hash: [u8; 32],

    /// The authority of the miner who submitted the best hash.
    pub best_hash_miner: Pubkey,

    /// The slot at which the block starts trading.
    pub start_slot: u64,

    /// The slot at which the block ends trading.
    pub end_slot: u64,

    /// The hash of the end slot, provided by solana, used for random number generation.
    pub slot_hash: [u8; 32],

    /// The total amount of hashpower bought in the block.
    pub total_hashpower: u64,
}

impl Block {
    pub fn pda(&self) -> (Pubkey, u8) {
        block_pda(self.id)
    }
}

account!(OreAccount, Block);
