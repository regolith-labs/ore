use bytemuck::{Pod, Zeroable};
use ore_utils::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MineEvent {
    pub difficulty: u64,
    pub reward: u64,
    pub timing: i64,
}

event!(MineEvent);
