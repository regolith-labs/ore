use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::OreAccountV4;

use super::OreAccountV1;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct TreasuryV1 {
    // The amount of SOL collected for buy-bury operations.
    pub balance: u64,

    /// Buffer a (placeholder)
    pub buffer_a: u64,

    /// The amount of ORE in the motherlode rewards pool.
    pub motherlode: u64,

    /// The cumulative ORE distributed to miners, divided by the total unclaimed ORE at the time of distribution.
    pub miner_rewards_factor: Numeric,

    /// The cumulative ORE distributed to stakers, divided by the total stake at the time of distribution.
    #[deprecated(since = "3.8.0", note = "Staking has moved to ore-stake program")]
    pub stake_rewards_factor: Numeric,

    /// Buffer b (placeholder)
    pub buffer_b: u64,

    /// The current total amount of refined ORE mining rewards.
    pub total_refined: u64,

    /// The current total amount of ORE staking deposits.
    #[deprecated(since = "3.8.0", note = "Staking has moved to ore-stake program")]
    pub total_staked: u64,

    /// The current total amount of unclaimed ORE mining rewards.
    pub total_unclaimed: u64,
}

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct TreasuryV4 {
    /// The amount of ORE in the motherlode rewards pool.
    pub motherlode: u64,

    /// The cumulative ORE distributed to miners, divided by the total unclaimed ORE at the time of distribution.
    pub rewards_factor: Numeric,

    /// The current total amount of refined ORE mining rewards.
    pub total_refined: u64,

    /// The current total amount of unrefined ORE mining rewards.
    pub total_unrefined: u64,
}

account!(OreAccountV1, TreasuryV1);
account!(OreAccountV4, TreasuryV4);

pub enum Treasury {
    TreasuryV1(TreasuryV1),
    TreasuryV4(TreasuryV4),
}

#[allow(deprecated)]
impl Treasury {
    pub fn balance(&self) -> u64 {
        match self {
            Treasury::TreasuryV1(t) => t.balance,
            Treasury::TreasuryV4(_) => 0,
        }
    }

    pub fn buffer_a(&self) -> u64 {
        match self {
            Treasury::TreasuryV1(t) => t.buffer_a,
            Treasury::TreasuryV4(_) => 0,
        }
    }

    pub fn motherlode(&self) -> u64 {
        match self {
            Treasury::TreasuryV1(t) => t.motherlode,
            Treasury::TreasuryV4(t) => t.motherlode,
        }
    }

    pub fn miner_rewards_factor(&self) -> Numeric {
        match self {
            Treasury::TreasuryV1(t) => t.miner_rewards_factor,
            Treasury::TreasuryV4(t) => t.rewards_factor,
        }
    }

    pub fn stake_rewards_factor(&self) -> Numeric {
        match self {
            Treasury::TreasuryV1(t) => t.stake_rewards_factor,
            Treasury::TreasuryV4(_) => Numeric::ZERO,
        }
    }

    pub fn buffer_b(&self) -> u64 {
        match self {
            Treasury::TreasuryV1(t) => t.buffer_b,
            Treasury::TreasuryV4(_) => 0,
        }
    }

    pub fn total_refined(&self) -> u64 {
        match self {
            Treasury::TreasuryV1(t) => t.total_refined,
            Treasury::TreasuryV4(t) => t.total_refined,
        }
    }

    pub fn total_staked(&self) -> u64 {
        match self {
            Treasury::TreasuryV1(t) => t.total_staked,
            Treasury::TreasuryV4(_) => 0,
        }
    }

    pub fn total_unclaimed(&self) -> u64 {
        match self {
            Treasury::TreasuryV1(t) => t.total_unclaimed,
            Treasury::TreasuryV4(t) => t.total_unrefined,
        }
    }
}
