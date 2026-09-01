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

    // Admin
    Buyback = 13,
    Bury = 24,
    Wrap = 14,
    UpdateProtocolConfig = 15,
    NewVar = 19,
    Liq = 25,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstructionV2 {
    AutomateV2 = 0,
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
pub struct AutomateV2 {
    pub amount: [u8; 8],
    pub deposit: [u8; 8],
    pub fee: [u8; 8],
    pub mask: [u8; 8],
    pub strategy: u8,
    pub reload: [u8; 8],
    pub conditions: [u8; 24],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimSOL {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimORE {
    pub bps: [u8; 8],
}

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
pub struct UpdateProtocolConfig {
    pub admin: [u8; 32],
    pub fee_collector: [u8; 32],
    pub fee_rate: [u8; 8],
    pub intermission_slots: [u8; 8],
    pub round_slots: [u8; 8],
    pub entropy_var_address: [u8; 32],
    pub entropy_program_id: [u8; 32],
}

instruction!(OreInstruction, Automate);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Checkpoint);
instruction!(OreInstruction, ClaimSOL);
instruction!(OreInstruction, ClaimORE);
instruction!(OreInstruction, Deploy);
instruction!(OreInstruction, Log);
instruction!(OreInstruction, Wrap);
instruction!(OreInstruction, Buyback);
instruction!(OreInstruction, Bury);
instruction!(OreInstruction, Reset);
instruction!(OreInstruction, NewVar);
instruction!(OreInstruction, UpdateProtocolConfig);
instruction!(OreInstructionV2, AutomateV2);
