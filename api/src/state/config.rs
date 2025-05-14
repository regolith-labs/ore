use steel::*;

use super::{OldOreAccount, OreAccount};

/// Config is a singleton account which manages program global variables.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {
    /// The timestamp of the last reset.
    pub last_reset_at: i64,

    /// The best difficulty score of this epoch.
    pub best_hash: [u8; 32],

    /// The proof of the best submitted hash of this epoch.
    pub best_proof: Pubkey,

    /// The challenge of this epoch.
    pub challenge: [u8; 32],

    /// The target emissions rate in ORE/min.
    pub block_reward: u64,
}

/// Config is a singleton account which manages program global variables.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct OldConfig {
    /// The base reward rate paid out for a hash of minimum difficulty.
    pub base_reward_rate: u64,

    /// The timestamp of the last reset.
    pub last_reset_at: i64,

    /// The minimum accepted difficulty.
    pub min_difficulty: u64,

    /// The target emissions rate in ORE/min.
    pub target_emmissions_rate: u64,
}

account!(OreAccount, Config);
account!(OldOreAccount, OldConfig);
