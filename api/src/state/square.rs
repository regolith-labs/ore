use steel::*;

use crate::state::square_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Square {
    /// The count of miners on this square.
    pub count: [u64; 25],

    /// The deployments of all players.
    pub deployed: [[u64; 16]; 25],

    /// The miners in each square.
    pub miners: [[Pubkey; 16]; 25],
}

impl Square {
    pub fn pda() -> (Pubkey, u8) {
        square_pda()
    }
}

account!(OreAccount, Square);
