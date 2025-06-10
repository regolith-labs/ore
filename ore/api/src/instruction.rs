use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    Open = 0,
    Close = 1,
    Commit = 2,
    Decommit = 3,
    Deposit = 4,
    Mine = 5,
    Swap = 6,
    Withdraw = 7,
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
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Decommit {
    pub amount: [u8; 8],
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
pub struct Withdraw {
    pub amount: [u8; 8],
}

instruction!(OreInstruction, Open);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Commit);
instruction!(OreInstruction, Decommit);
instruction!(OreInstruction, Deposit);
instruction!(OreInstruction, Mine);
instruction!(OreInstruction, Swap);
instruction!(OreInstruction, Withdraw);
