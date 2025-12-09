use serde::{Deserialize, Serialize};
use steel::*;

pub enum OreEvent {
    Reset = 0,
    Bury = 1,
    Deploy = 2,
    Liq = 3,
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
pub struct DeployEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the deployer.
    pub authority: Pubkey,

    /// The amount of SOL deployed per square.
    pub amount: u64,

    /// The mask of the squares deployed to.
    pub mask: u64,

    /// The round id.
    pub round_id: u64,

    /// The signer of the deployer.
    pub signer: Pubkey,

    /// The strategy used by the autominer (u64::MAX if manual).
    pub strategy: u64,

    /// The total number of squares deployed to.
    pub total_squares: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct LiqEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The amount of SOL sent to the liq manager.
    pub sol_amount: u64,

    /// The recipient of the SOL.
    pub recipient: Pubkey,

    /// The timestamp of the event.
    pub ts: i64,
}

event!(ResetEvent);
event!(BuryEvent);
event!(DeployEvent);
event!(LiqEvent);
