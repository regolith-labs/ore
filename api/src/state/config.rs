use steel::*;

use super::OreAccount;

/// Config is a singleton account which manages program global variables.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {
    /// The base reward rate paid out for a hash of minimum difficulty.
    pub base_reward_rate: u64,

    /// The timestamp of the last reset.
    pub last_reset_at: i64,

    /// The minimum accepted difficulty.
    pub min_difficulty: u64,

    /// ORE / min
    pub target_emmissions_rate: u64,

    ///
    pub next_emmissions_rate_update: u64,
}

account!(OreAccount, Config);
