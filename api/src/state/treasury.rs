use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::OreAccountV4;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Treasury {
    /// The amount of ORE in the motherlode rewards pool.
    pub motherlode: u64,

    /// The cumulative ORE distributed to miners, divided by the total unclaimed ORE at the time of distribution.
    /// TODO: rename to rewards_factor
    pub miner_rewards_factor: Numeric,

    /// The current total amount of refined ORE mining rewards.
    pub total_refined: u64,

    /// The current total amount of unrefined ORE mining rewards.
    /// TODO rename to total_unrefined
    pub total_unclaimed: u64,
}

account!(OreAccountV4, Treasury);
