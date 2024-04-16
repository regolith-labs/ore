use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::{
    impl_account_from_bytes, impl_to_bytes,
    state::Hash,
    utils::{AccountDiscriminator, Discriminator},
};

/// Config is a singleton account which manages admin configurable variables.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, ShankAccount, Zeroable)]
pub struct Config {
    /// The admin authority with permission to update the difficulty.
    pub admin: Pubkey,

    /// The hash difficulty.
    pub difficulty: Hash,
}

impl Discriminator for Config {
    fn discriminator() -> AccountDiscriminator {
        AccountDiscriminator::Config
    }
}

impl_to_bytes!(Config);
impl_account_from_bytes!(Config);
