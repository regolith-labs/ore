use serde::{Deserialize, Serialize};
use steel::*;

use super::FpowAccount;

/// Treasury is a singleton account which is the mint authority for the fPOW token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Treasury {
    // The amount of ALGO collected for buy-bury operations.
    pub balance: u64,

    /// Buffer a (placeholder)
    pub buffer_a: u64,

    /// The amount of fPOW in the motherlode rewards pool.
    pub motherlode: u64,

    /// The cumulative fPOW distributed to miners, divided by the total unclaimed fPOW at the time of distribution.
    pub miner_rewards_factor: Numeric,

    /// The cumulative fPOW distributed to stakers, divided by the total stake at the time of distribution.
    pub stake_rewards_factor: Numeric,

    /// Buffer b (placeholder)
    pub buffer_b: u64,

    /// The current total amount of refined fPOW mining rewards.
    pub total_refined: u64,

    /// The current total amount of fPOW staking deposits.
    pub total_staked: u64,

    /// The current total amount of unclaimed fPOW mining rewards.
    pub total_unclaimed: u64,
}

account!(FpowAccount, Treasury);
