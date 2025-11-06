use serde::{Deserialize, Serialize};
use steel::*;
pub enum OreEvent {
    Reset = 0,
    Bury = 1,
    Motherlode = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct ResetEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The block that was opened for trading.
    pub round_id: u64,

    /// The start slot of the next block.
    pub start_slot: u64,

    /// The end slot of the next block.
    pub end_slot: u64,

    /// The winning square of the round.
    pub winning_square: u64,

    /// The top miner of the round.
    pub top_miner: Pubkey,

    /// The number of miners on the winning square.
    pub num_winners: u64,

    /// The amount of ORE payout for the motherlode.
    pub motherlode: u64,

    /// The total amount of SOL prospected in the round.
    pub total_deployed: u64,

    /// The total amount of SOL put in the ORE vault.
    pub total_vaulted: u64,

    /// The total amount of SOL won by miners for the round.
    pub total_winnings: u64,

    /// The total amount of ORE minted for the round.
    pub total_minted: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct BuryEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The amount of ORE buried.
    pub ore_buried: u64,

    /// The amount of ORE shared with stakers.
    pub ore_shared: u64,

    /// The amount of SOL swapped.
    pub sol_amount: u64,

    /// The new circulating supply of ORE.
    pub new_circulating_supply: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct MotherlodeEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The amount of ORE minted to the motherlode.
    pub amount: u64,

    /// The round id.
    pub round_id: u64,

    /// The number of winners.
    pub num_miners: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

event!(ResetEvent);
event!(BuryEvent);
