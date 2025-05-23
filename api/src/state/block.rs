use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Block {
    /// The current round.
    pub current_round: u64,

    /// The cumulative amount of SOL risked in the current round, up to and including this bet.
    pub total_bets: u64,

    /// The number of bets made in the current round.
    pub bet_count: u64,

    /// The time time the current round started.
    pub started_at: u64,

    /// The slot at which the current round ends.
    pub ends_at: u64,

    /// Whether or not the current round has ended.
    pub payed_out: u64,

    /// The mint used to track wagers of the current round.
    pub mint: Pubkey,

    /// The amount of ORE to distribute to the winner.
    pub reward: u64,

    /// The noise used for the current round.
    pub noise: [u8; 32],
}

account!(OreAccount, Block);
