use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Wager {
    /// The amount bet in this wager.
    pub amount: u64,

    /// The signer authorized to use this wager.
    pub authority: Pubkey,

    /// The cumulative amount bet in the current round prior to this wager.
    pub cumulative_sum: u64,

    /// The current round this wager is for.
    pub round: u64,

    /// The ID of the wager, used for provably fair randomness.
    pub seed: [u8; 32],

    /// The timestamp of the wager.
    pub timestamp: u64,
}

account!(OreAccount, Wager);
