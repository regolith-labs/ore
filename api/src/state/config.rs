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

    /// Buffer for possible future use.
    pub _buffer: [u8; 8],
}

account!(OreAccount, Config);
