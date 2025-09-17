use steel::*;

use crate::state::board_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Board {
    /// The commits for the round.
    pub commits: [u64; 25],

    /// The round number.
    pub id: u64,

    /// The timestamp at which the block starts mining.
    pub start_at: i64,

    /// The slot at which the block starts trading.
    pub start_slot: u64,

    /// The slot at which the block ends trading.
    pub end_slot: u64,

    /// The hash of the end slot, provided by solana, used for random number generation.
    pub slot_hash: [u8; 32],

    /// The total amount of ORE burned for the round.
    pub total_burned: u64,

    /// The total amount of ORE committed for the round.
    pub total_commits: u64,

    /// The total amount of ORE won by miners for the round.
    pub total_winnings: u64,
}

impl Board {
    pub fn pda(&self) -> (Pubkey, u8) {
        board_pda()
    }
}

account!(OreAccount, Board);
