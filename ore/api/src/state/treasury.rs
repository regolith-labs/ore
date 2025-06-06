use steel::*;

use super::OreAccount;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {
    /// The total amount of ORE that has been staked.
    pub total_stake: u64,
}

account!(OreAccount, Treasury);
