use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // Miner
    Automate = 0,
    Checkpoint = 2,
    ClaimSOL = 3,
    ClaimORE = 4,
    Close = 5,
    Deploy = 6,
    Log = 8,
    Reset = 9,
    RecycleSOL = 21,

    // Staker
    Deposit = 10,
    Withdraw = 11,
    ClaimYield = 12,

    // Admin
    Bury = 13,
    Wrap = 14,
    SetAdmin = 15,
    SetFeeCollector = 16,
    SetSwapProgram = 17,
    SetVarAddress = 18,
    NewVar = 19,
    SetAdminFee = 20,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Automate {
    pub amount: [u8; 8],
    pub deposit: [u8; 8],
    pub fee: [u8; 8],
    pub mask: [u8; 8],
    pub strategy: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimSOL {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimORE {}

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
pub struct Wrap {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Bury {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RecycleSOL {}

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

instruction!(OreInstruction, Automate);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Checkpoint);
instruction!(OreInstruction, ClaimSOL);
instruction!(OreInstruction, ClaimORE);
instruction!(OreInstruction, RecycleSOL);
instruction!(OreInstruction, Deploy);
instruction!(OreInstruction, Log);
instruction!(OreInstruction, Wrap);
instruction!(OreInstruction, Bury);
instruction!(OreInstruction, Reset);
instruction!(OreInstruction, SetAdmin);
instruction!(OreInstruction, SetFeeCollector);
instruction!(OreInstruction, Deposit);
instruction!(OreInstruction, Withdraw);
instruction!(OreInstruction, ClaimYield);
instruction!(OreInstruction, NewVar);
instruction!(OreInstruction, SetAdminFee);
instruction!(OreInstruction, SetSwapProgram);
instruction!(OreInstruction, SetVarAddress);
