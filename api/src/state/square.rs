use steel::*;

use crate::state::square_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Square {
    /// The count of miners on this square.
    pub count: u64,

    /// The commits for the round.
    pub id: u64,

    /// The round number.
    pub round_id: u64,

    /// The miners on this square.
    pub miners: [Pubkey; 16],
}

impl Square {
    pub fn pda(&self) -> (Pubkey, u8) {
        square_pda(self.id)
    }
}

account!(OreAccount, Square);
