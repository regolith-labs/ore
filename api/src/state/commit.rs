use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Commit {
    /// The amount deployed in this commit.
    pub amount: u64,

    /// The signer authorized to use this commit.
    pub authority: Pubkey,

    /// The cumulative amount deployed in the current round prior to this commit.
    pub cumulative_sum: u64,

    /// The current round this commit is for.
    pub round: u64,

    /// The ID of the commit, used for provably fair randomness.
    pub seed: [u8; 32],

    /// The timestamp of the commit.
    pub timestamp: u64,
}

account!(OreAccount, Commit);
