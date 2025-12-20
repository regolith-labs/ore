use serde::{Deserialize, Serialize};

use crate::state::board_box_name;

/// Board state - tracks the current game round
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Board {
    /// The current round number.
    pub round_id: u64,

    /// The Algorand round at which the current round starts mining.
    pub start_round: u64,

    /// The Algorand round at which the current round ends mining.
    pub end_round: u64,

    /// The current epoch id.
    pub epoch_id: u64,
}

impl Board {
    pub fn box_name() -> Vec<u8> {
        board_box_name()
    }
}
