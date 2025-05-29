use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct BuryEvent {
    pub amount: u64,
    pub ts: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct DeployEvent {
    pub authority: Pubkey,
    pub amount: u64,
    pub ts: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct PayoutEvent {
    pub authority: Pubkey,
    pub amount: u64,
    pub ts: u64,
}

event!(BuryEvent);
event!(DeployEvent);
event!(PayoutEvent);
