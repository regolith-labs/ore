use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // User
    Open = 0,
    Close = 1,
    Commit = 2,
    Deposit = 3,
    Mine = 4,
    Swap = 5,
    Uncommit = 6,
    Withdraw = 7,

    // Admin
    SetAdmin = 10,
    SetFeeCollector = 9,
    SetFeeRate = 8,
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
pub struct Commit {
    pub amount: [u8; 8],
    pub executor: [u8; 32],
    pub fee: [u8; 8],
    pub seed: [u8; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Deposit {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Mine {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Swap {
    pub amount: [u8; 8],
    pub direction: u8,
    pub precision: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Uncommit {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Withdraw {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetAdmin {
    pub admin: [u8; 32],
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

instruction!(OreInstruction, Open);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Commit);
instruction!(OreInstruction, Deposit);
instruction!(OreInstruction, Mine);
instruction!(OreInstruction, Swap);
instruction!(OreInstruction, Uncommit);
instruction!(OreInstruction, Withdraw);
instruction!(OreInstruction, SetAdmin);
instruction!(OreInstruction, SetFeeCollector);
instruction!(OreInstruction, SetFeeRate);
