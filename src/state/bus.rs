use bytemuck::{Pod, Zeroable};

use crate::{impl_account_from_bytes, impl_to_bytes};

use super::{AccountDiscriminator, Discriminator};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Bus {
    /// The bump of the bus account PDA.
    pub bump: u32,

    /// The ID of the bus account.
    pub id: u32,

    /// The quantity of rewards this bus can issue in the current epoch epoch.
    pub available_rewards: u64,
}

impl Discriminator for Bus {
    fn discriminator() -> super::AccountDiscriminator {
        AccountDiscriminator::Bus
    }
}

impl_to_bytes!(Bus);
impl_account_from_bytes!(Bus);
