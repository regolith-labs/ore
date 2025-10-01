use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // Miner
    Automate = 0,
    Boost = 1,
    Checkpoint = 2,
    ClaimSOL = 3,
    ClaimORE = 4,
    Deploy = 5,
    Initialize = 6,
    Log = 7,
    Reset = 8,

    // Staker
    Deposit = 9,
    Withdraw = 10,
    ClaimYield = 11,

    // Admin
    Bury = 12,
    Wrap = 13,
    SetAdmin = 14,
    SetFeeCollector = 15,

    // Seeker
    ClaimSeeker = 16,
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
pub struct Boost {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimSOL {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimORE {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Deploy {
    pub amount: [u8; 8],
    pub squares: [u8; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Initialize {}

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
pub struct Bury {
    pub min_amount_out: [u8; 8],
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
pub struct ClaimYield {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimSeeker {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Checkpoint {}

instruction!(OreInstruction, Automate);
instruction!(OreInstruction, Boost);
instruction!(OreInstruction, Checkpoint);
instruction!(OreInstruction, ClaimSOL);
instruction!(OreInstruction, ClaimORE);
instruction!(OreInstruction, Deploy);
instruction!(OreInstruction, Initialize);
instruction!(OreInstruction, Log);
instruction!(OreInstruction, Wrap);
instruction!(OreInstruction, Bury);
instruction!(OreInstruction, Reset);
instruction!(OreInstruction, SetAdmin);
instruction!(OreInstruction, SetFeeCollector);
instruction!(OreInstruction, Deposit);
instruction!(OreInstruction, Withdraw);
instruction!(OreInstruction, ClaimYield);
instruction!(OreInstruction, ClaimSeeker);
