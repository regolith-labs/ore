use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;

use crate::utils::{impl_account_from_bytes, impl_to_bytes, Discriminator};

use super::AccountDiscriminator;

/// Config is a singleton account which manages admin configurable variables.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, ShankAccount, Zeroable)]
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

impl_to_bytes!(Config);
impl_account_from_bytes!(Config);
