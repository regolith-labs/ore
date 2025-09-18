use steel::*;

use crate::state::treasury_pda;

use super::OreAccount;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {
    pub balance: u64,
}

account!(OreAccount, Treasury);
