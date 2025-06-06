use steel::*;

use super::OreDelegateAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Delegate {
    /// The authority of the delegate.
    pub authority: Pubkey,

    /// The number of hash tokens deposited for mining.
    pub balance: u64,

    /// The block these hash tokens are associated with.
    pub block_id: u64,

    /// The fee to payout per crank (lamports).
    pub fee: u64,
}

account!(OreDelegateAccount, Delegate);
