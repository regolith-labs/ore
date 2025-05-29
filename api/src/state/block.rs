use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Block {
    /// The cumulative amount deployed in the current round.
    pub cumulative_sum: u64,

    /// The current round.
    pub current_round: u64,

    /// The slot at which the current round ends.
    pub ends_at: u64,

    /// The mint used for commits of the current round.
    pub mint: Pubkey,

    /// The noise used for the current round for provably fair randomness.
    pub noise: [u8; 32],

    /// Whether or not the current round has paid out.
    pub paid: u64,

    /// The amount of ORE to distribute to the winner.
    pub reward: u64,

    /// The time the current round started at.
    pub started_at: u64,

    /// The number of commits made in the current round.
    pub total_commits: u64,
}

account!(OreAccount, Block);
