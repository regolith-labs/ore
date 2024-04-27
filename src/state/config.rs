use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

// TODO next_min_difficulty: Option<u8>, update on reset

use crate::{
    impl_account_from_bytes, impl_to_bytes,
    utils::{AccountDiscriminator, Discriminator},
};

/// Config is a singleton account which manages admin configurable variables.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, ShankAccount, Zeroable)]
pub struct Config {
    /// The admin authority with permission to update the difficulty.
    pub admin: Pubkey,

    /// The base reward rate paid out for a hash of minimum difficulty.
    pub base_reward_rate: u64,

    /// The minimum accepted difficulty.
    pub min_difficulty: u32,

    /// Is mining paused.
    pub paused: u32,
}

impl Discriminator for Config {
    fn discriminator() -> AccountDiscriminator {
        AccountDiscriminator::Config
    }
}

impl_to_bytes!(Config);
impl_account_from_bytes!(Config);
