use steel::*;

use crate::state::board_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Game {
    /// The current round number.
    pub round_id: u64,

    /// The timestamp at which the current round starts mining.
    pub start_at: i64,

    /// The slot at which the current round starts mining.
    pub start_slot: u64,

    /// The slot at which the current round ends mining.
    pub end_slot: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Round {
    /// The round number.
    pub id: u64,

    /// The amount of SOL deployed in each square.
    pub deployed: [u64; 25],

    /// The hash of the end slot, provided by solana, used for random number generation.
    pub slot_hash: [u8; 32],

    /// The slot at which claims for this round account end.
    pub expires_at: i64,

    /// The top miner of the round.
    pub top_miner: Pubkey,

    /// The total amount of SOL deployed in the round.
    pub total_deployed: u64,

    /// The total amount of SOL put in the ORE vault.
    pub total_vaulted: u64,

    /// The total amount of SOL won by miners for the round.
    pub total_winnings: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Move {
    /// The authority of the move.
    pub authority: Pubkey,

    /// The amount of SOL deployed in each square.
    pub deployed: [u64; 25],

    /// The round number.
    pub round_id: u64,
}

// impl Board {
//     pub fn pda(&self) -> (Pubkey, u8) {
//         board_pda()
//     }
// }

// account!(OreAccount, Board);
