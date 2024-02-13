use bytemuck::{Pod, Zeroable};

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

impl Bus {
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
