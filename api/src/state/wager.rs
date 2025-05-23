use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Wager {
    /// The signer authorized to use this wager.
    pub authority: Pubkey,

    /// The current round this miner is betting in.
    pub round: u64,

    /// The ID of the bet.
    pub id: u64,

    /// The quantity of SOL this miner has bet in the current round.
    pub amount: u64,

    /// The cumulative amount of SOL bet in the current round, up to and including this wager.
    pub cumulative_bets: u64,

    /// The timestamp of the wager.
    pub timestamp: u64,
}

account!(OreAccount, Wager);
