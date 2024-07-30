use bytemuck::{Pod, Zeroable};

use crate::utils::{impl_account_from_bytes, impl_to_bytes, Discriminator};

use super::AccountDiscriminator;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {}

impl Discriminator for Treasury {
    fn discriminator() -> u8 {
        AccountDiscriminator::Treasury.into()
    }
}

impl_to_bytes!(Treasury);
impl_account_from_bytes!(Treasury);
