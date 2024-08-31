use bytemuck::{Pod, Zeroable};

use crate::utils::{impl_account_from_bytes, impl_to_bytes, Discriminator};

use super::AccountDiscriminator;

/// Config is a singleton account which manages program coal minting variables.
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

impl Discriminator for Config {
    fn discriminator() -> u8 {
        AccountDiscriminator::Config.into()
    }
}

/// Config is a singleton account which manages program coal minting variables.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct WoodConfig {
    /// The base reward rate paid out for a hash of minimum difficulty.
    pub base_reward_rate: u64,

    /// The timestamp of the last reset.
    pub last_reset_at: i64,

    /// The minimum accepted difficulty.
    pub min_difficulty: u64,

    /// The largest known stake balance on the network from the last epoch.
    pub top_balance: u64,

    /// The current epoch emission rate for the program.
    pub total_epoch_rewards: u64,
}

impl Discriminator for WoodConfig {
    fn discriminator() -> u8 {
        AccountDiscriminator::WoodConfig.into()
    }
}

impl_to_bytes!(Config);
impl_account_from_bytes!(Config);
impl_to_bytes!(WoodConfig);
impl_account_from_bytes!(WoodConfig);
