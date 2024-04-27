use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;

use crate::{
    impl_account_from_bytes, impl_to_bytes,
    utils::{AccountDiscriminator, Discriminator},
};

/// Treasury is a singleton account which manages all program wide variables.
/// It is the mint authority for the Ore token and also the authority of the program-owned token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, ShankAccount, Zeroable)]
pub struct Treasury {
    /// The bump of the treasury account PDA, for signing CPIs.
    pub bump: u64,

    /// The total lifetime claimed rewards of the program.
    pub total_claimed_rewards: u64,
}

impl Discriminator for Treasury {
    fn discriminator() -> AccountDiscriminator {
        AccountDiscriminator::Treasury
    }
}

impl_to_bytes!(Treasury);
impl_account_from_bytes!(Treasury);
