use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // User
    Claim = 0,
    Log = 1,
    Mine = 2,
    Swap = 3,
    Initialize = 4,
    Open = 5,
    Close = 6,
    Reset = 7,

    // Admin
    SetAdmin = 8,
    SetBlockDuration = 9,
    SetFeeCollector = 10,
    SetFeeRate = 11,
    SetSniperFeeDuration = 12,

    // Seeker
    ClaimSeeker = 13,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Claim {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Open {
    pub id: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Reset {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Initialize {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Log {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Mine {
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Swap {
    pub amount: [u8; 8],
    pub direction: u8,
    pub precision: u8,
    pub seed: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Uncommit {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetAdmin {
    pub admin: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetBlockDuration {
    pub block_duration: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetFeeCollector {
    pub fee_collector: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetFeeRate {
    pub fee_rate: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetSniperFeeDuration {
    pub sniper_fee_duration: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimSeeker {}

instruction!(OreInstruction, Claim);
instruction!(OreInstruction, Open);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Reset);
instruction!(OreInstruction, Initialize);
instruction!(OreInstruction, Log);
instruction!(OreInstruction, Mine);
instruction!(OreInstruction, Swap);
instruction!(OreInstruction, SetAdmin);
instruction!(OreInstruction, SetBlockDuration);
instruction!(OreInstruction, SetFeeCollector);
instruction!(OreInstruction, SetFeeRate);
instruction!(OreInstruction, SetSniperFeeDuration);
instruction!(OreInstruction, ClaimSeeker);
