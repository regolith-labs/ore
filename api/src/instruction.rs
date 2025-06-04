use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // Block
    Open = 0,
    Close = 1,
    Mine = 2,

    // Market
    Buy = 3,
    Sell = 4,

    // Stake
    Deposit = 5,
    Withdraw = 6,
    Free = 7,
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
pub struct Mine {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Buy {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Sell {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Deposit {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Withdraw {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Free {}

instruction!(OreInstruction, Open);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Mine);
instruction!(OreInstruction, Buy);
instruction!(OreInstruction, Sell);
instruction!(OreInstruction, Deposit);
instruction!(OreInstruction, Withdraw);
instruction!(OreInstruction, Free);
