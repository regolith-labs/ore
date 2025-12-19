use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum FpowInstruction {
    // Miner
    Automate = 0,
    Checkpoint = 2,
    ClaimALGO = 3,
    ClaimFPOW = 4,
    Close = 5,
    Deploy = 6,
    Log = 8,
    Reset = 9,
    ReloadALGO = 21,

    // Staker
    Deposit = 10,
    Withdraw = 11,
    ClaimYield = 12,
    CompoundYield = 22,

    // Admin
    Buyback = 13,
    Bury = 24,
    Wrap = 14,
    SetAdmin = 15,
    NewVar = 19,
    Liq = 25,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Automate {
    pub amount: [u8; 8],
    pub deposit: [u8; 8],
    pub fee: [u8; 8],
    pub mask: [u8; 8],
    pub strategy: u8,
    pub reload: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimALGO {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimFPOW {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Deploy {
    pub amount: [u8; 8],
    pub squares: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Log {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Reset {}

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
pub struct Wrap {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Buyback {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Bury {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ReloadALGO {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Deposit {
    pub amount: [u8; 8],
    pub compound_fee: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Withdraw {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimYield {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Checkpoint {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct NewVar {
    pub id: [u8; 8],
    pub commit: [u8; 32],
    pub samples: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetAdminFee {
    pub admin_fee: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetSwapProgram {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct SetVarAddress {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Liq {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CompoundYield {}

instruction!(FpowInstruction, Automate);
instruction!(FpowInstruction, Close);
instruction!(FpowInstruction, Checkpoint);
instruction!(FpowInstruction, ClaimALGO);
instruction!(FpowInstruction, ClaimFPOW);
instruction!(FpowInstruction, ReloadALGO);
instruction!(FpowInstruction, Deploy);
instruction!(FpowInstruction, Log);
instruction!(FpowInstruction, Wrap);
instruction!(FpowInstruction, Buyback);
instruction!(FpowInstruction, Bury);
instruction!(FpowInstruction, Reset);
instruction!(FpowInstruction, SetAdmin);
instruction!(FpowInstruction, Deposit);
instruction!(FpowInstruction, Withdraw);
instruction!(FpowInstruction, ClaimYield);
instruction!(FpowInstruction, NewVar);
instruction!(FpowInstruction, Liq);
instruction!(FpowInstruction, CompoundYield);
