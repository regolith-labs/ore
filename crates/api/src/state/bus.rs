use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;

use crate::{
    impl_account_from_bytes, impl_to_bytes,
    utils::{AccountDiscriminator, Discriminator},
};

/// Bus accounts are responsible for distributing mining rewards.
/// There are 8 busses total to minimize write-lock contention and allow for parallel mine operations.
/// Every epoch, the bus account rewards counters are topped up to 0.25 ORE each (2 ORE split amongst 8 busses).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, ShankAccount, Zeroable)]
pub struct Bus {
    /// The ID of the bus account.
    pub id: u64,

    /// The quantity of rewards this bus can issue in the current epoch epoch.
    pub rewards: u64,
}

impl Discriminator for Bus {
    fn discriminator() -> AccountDiscriminator {
        AccountDiscriminator::Bus
    }
}

impl_to_bytes!(Bus);
impl_account_from_bytes!(Bus);
