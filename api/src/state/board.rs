use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{board_pda, stats_pda, OreAccountV4};

use super::OreAccountV1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct BoardV1 {
    /// The current round number.
    pub round_id: u64,

    /// The slot at which the current round starts mining.
    pub start_slot: u64,

    /// The slot at which the current round ends mining.
    pub end_slot: u64,

    /// The current epoch id.
    pub epoch_id: u64,
}

/// BoardV4 is a singleton account tracking global game state.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct BoardV4 {
    /// The current round number.
    pub round_id: u64,

    /// The slot at which the current round starts mining.
    pub start_slot: u64,

    /// The slot at which the current round ends mining.
    pub end_slot: u64,

    /// The exponential moving average of production cost (lamports per whole ORE).
    pub production_cost_ema: u64,
}

impl BoardV1 {
    pub fn pda(&self) -> (Pubkey, u8) {
        board_pda()
    }
}

impl BoardV4 {
    pub fn pda(&self) -> (Pubkey, u8) {
        stats_pda()
    }
}

account!(OreAccountV1, BoardV1);
account!(OreAccountV4, BoardV4);

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Board {
    V1(BoardV1),
    V4(BoardV4),
}

impl Board {
    pub fn round_id(&self) -> u64 {
        match self {
            Board::V1(b) => b.round_id,
            Board::V4(b) => b.round_id,
        }
    }

    pub fn start_slot(&self) -> u64 {
        match self {
            Board::V1(b) => b.start_slot,
            Board::V4(b) => b.start_slot,
        }
    }

    pub fn end_slot(&self) -> u64 {
        match self {
            Board::V1(b) => b.end_slot,
            Board::V4(b) => b.end_slot,
        }
    }

    pub fn epoch_id(&self) -> u64 {
        match self {
            Board::V1(b) => b.epoch_id,
            Board::V4(_) => 0,
        }
    }

    pub fn pda(&self) -> (Pubkey, u8) {
        match self {
            Board::V1(b) => b.pda(),
            Board::V4(b) => b.pda(),
        }
    }
}
