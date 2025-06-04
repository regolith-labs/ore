use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Market {
    /// The id of the block this market is associated with.
    pub block_id: u64,
}

// TODO Bonding curve stuff

account!(OreAccount, Market);
