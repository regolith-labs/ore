use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{board_pda, OreAccountV4};

/// BoardV4 is a singleton account tracking global game state.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Board {
    /// The current round number.
    pub round_id: u64,

    /// The slot at which the current round starts mining.
    pub start_slot: u64,

    /// The slot at which the current round ends mining.
    pub end_slot: u64,

    /// The exponential moving average of production cost (lamports per whole ORE).
    pub production_cost_ema: u64,
}

impl Board {
    pub fn pda(&self) -> (Pubkey, u8) {
        board_pda()
    }
}

account!(OreAccountV4, Board);
