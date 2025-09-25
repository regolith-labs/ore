use steel::*;

use super::OreAccount;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {
    // The amount of SOL collected for buy-bury operations.
    pub balance: u64,

    /// The amount of ORE in the motherlode.
    pub motherlode: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct TreasuryOLD {
    // The amount of SOL collected for buy-bury operations.
    pub balance: u64,
}

account!(OreAccount, Treasury);
