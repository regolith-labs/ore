use serde::{Deserialize, Serialize};

/// Event types emitted by the fPOW contract
pub enum FpowEvent {
    Reset = 0,
    Bury = 1,
    Deploy = 2,
    Liq = 3,
}

/// Event emitted when a new round starts
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ResetEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The block that was opened for trading.
    pub round_id: u64,

    /// The start round of the next block.
    pub start_round: u64,

    /// The end round of the next block.
    pub end_round: u64,

    /// The winning square of the round.
    pub winning_square: u64,

    /// The top miner of the round (Algorand address bytes).
    pub top_miner: [u8; 32],

    /// The number of miners on the winning square.
    pub num_winners: u64,

    /// The amount of fPOW payout for the motherlode.
    pub motherlode: u64,

    /// The total amount of ALGO prospected in the round.
    pub total_deployed: u64,

    /// The total amount of ALGO put in the fPOW vault.
    pub total_vaulted: u64,

    /// The total amount of ALGO won by miners for the round.
    pub total_winnings: u64,

    /// The total amount of fPOW minted for the round.
    pub total_minted: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

/// Event emitted when tokens are buried
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct BuryEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The amount of fPOW buried.
    pub fpow_buried: u64,

    /// The amount of fPOW shared with stakers.
    pub fpow_shared: u64,

    /// The amount of ALGO swapped.
    pub algo_amount: u64,

    /// The new circulating supply of fPOW.
    pub new_circulating_supply: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

/// Event emitted when a miner deploys
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DeployEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The authority of the deployer (Algorand address bytes).
    pub authority: [u8; 32],

    /// The amount of ALGO deployed per square.
    pub amount: u64,

    /// The mask of the squares deployed to.
    pub mask: u64,

    /// The round id.
    pub round_id: u64,

    /// The signer of the deployer (Algorand address bytes).
    pub signer: [u8; 32],

    /// The strategy used by the autominer (u64::MAX if manual).
    pub strategy: u64,

    /// The total number of squares deployed to.
    pub total_squares: u64,

    /// The timestamp of the event.
    pub ts: i64,
}

/// Event emitted for liquidity operations
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LiqEvent {
    /// The event discriminator.
    pub disc: u64,

    /// The amount of ALGO sent to the liq manager.
    pub algo_amount: u64,

    /// The recipient of the ALGO (Algorand address bytes).
    pub recipient: [u8; 32],

    /// The timestamp of the event.
    pub ts: i64,
}
