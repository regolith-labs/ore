use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct MineEvent {
    pub difficulty: u64,
    pub reward: u64,
    pub timing: i64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct BoostEvent {
    pub mint: Pubkey,
    pub reward: u64,
}

event!(MineEvent);
event!(BoostEvent);
