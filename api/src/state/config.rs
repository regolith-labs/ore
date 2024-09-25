use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use steel::*;

use crate::consts::CONFIG;

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

    /// The largest known stake balance on the network from the last epoch.
    pub top_balance: u64,
}

/// Derive the PDA of the config account.
pub fn config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CONFIG], &crate::id())
}

account!(OreAccount, Config);
