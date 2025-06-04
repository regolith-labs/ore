use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Receipt {
    /// The authority of this receipt account.
    pub authority: Pubkey,

    /// The amount of ORE this miner has deployed in the hashpower market corresponding to the block id.
    pub amount: u64,

    /// The id of the block this receipt is associated with.
    pub block_id: u64,
}

account!(OreAccount, Receipt);
