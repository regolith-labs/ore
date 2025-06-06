use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreDelegateInstruction {
    Deposit = 0,
    Withdraw = 1,
    Crank = 2,
    Payout = 3,
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
pub struct Crank {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Payout {}

instruction!(OreDelegateInstruction, Deposit);
instruction!(OreDelegateInstruction, Withdraw);
instruction!(OreDelegateInstruction, Crank);
instruction!(OreDelegateInstruction, Payout);
