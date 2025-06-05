use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Market {
    /// The id of the block this market is associated with.
    pub id: u64,

    /// Mint of the hash token.
    pub mint: Pubkey,
}

// TODO Bonding curve stuff

account!(OreAccount, Market);
