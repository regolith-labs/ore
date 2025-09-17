use steel::*;

use crate::state::board_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Board {
    /// The round number.
    pub id: u64,

    /// The prospects for the round.
    pub prospects: [u64; 25],

    /// The timestamp at which the block starts mining.
    pub start_at: i64,

    /// The slot at which the block starts trading.
    pub start_slot: u64,

    /// The slot at which the block ends trading.
    pub end_slot: u64,

    /// The hash of the end slot, provided by solana, used for random number generation.
    pub slot_hash: [u8; 32],

    /// The top miner of the round.
    pub top_miner: Pubkey,

    /// The total amount of SOL prospected in the round.
    pub total_prospects: u64,

    /// The total amount of SOL put in the ORE vault.
    pub total_vaulted: u64,

    /// The total amount of SOL won by miners for the round.
    pub total_winnings: u64,
}

impl Board {
    pub fn pda(&self) -> (Pubkey, u8) {
        board_pda()
    }
}

account!(OreAccount, Board);
