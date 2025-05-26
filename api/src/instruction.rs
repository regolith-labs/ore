use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // User
    Claim = 0,
    Bet = 1,
    Bury = 2,
    Close = 3,
    Payout = 4,
    Reset = 5,

    // Admin
    Initialize = 100,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Bet {
    pub amount: [u8; 8],
    pub seed: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Bury {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Claim {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Reset {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Payout {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Initialize {}

instruction!(OreInstruction, Claim);
instruction!(OreInstruction, Bet);
instruction!(OreInstruction, Bury);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Payout);
instruction!(OreInstruction, Reset);
instruction!(OreInstruction, Initialize);
