use bytemuck::{Pod, Zeroable};

use crate::impl_to_bytes;

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

impl_to_bytes!(Bus);
