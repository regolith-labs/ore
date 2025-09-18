use steel::*;

use crate::state::{treasury_pda, OreAccountOLD};

use super::OreAccount;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {
    pub balance: u64,
}

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct TreasuryOLD {}

impl Treasury {
    pub fn pda() -> (Pubkey, u8) {
        treasury_pda()
    }
}

account!(OreAccount, Treasury);
account!(OreAccountOLD, TreasuryOLD);
