use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // Mine
    Open = 0,
    Close = 1,
    Mine = 2,

    // Stake
    Deposit = 3,
    Withdraw = 4,
    Free = 5,

    // Trade
    Buy = 6,
    Sell = 7,
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
pub struct Deposit {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Withdraw {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Free {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Buy {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Sell {}

instruction!(OreInstruction, Open);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Mine);
instruction!(OreInstruction, Deposit);
instruction!(OreInstruction, Withdraw);
instruction!(OreInstruction, Free);
instruction!(OreInstruction, Buy);
instruction!(OreInstruction, Sell);
