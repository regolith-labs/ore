use serde::{Deserialize, Serialize};

/// Method selectors for fPOW Algorand application calls.
/// These correspond to the ABI method signatures in the PyTeal contract.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl FpowInstruction {
    /// Get the ABI method selector for this instruction
    pub fn method_selector(&self) -> &'static str {
        match self {
            FpowInstruction::Automate => "automate(uint64,uint64,uint64,uint64,uint8,uint64)void",
            FpowInstruction::Checkpoint => "checkpoint()void",
            FpowInstruction::ClaimALGO => "claim_algo()void",
            FpowInstruction::ClaimFPOW => "claim_fpow()void",
            FpowInstruction::Close => "close()void",
            FpowInstruction::Deploy => "deploy(uint64,uint32)void",
            FpowInstruction::Log => "log()void",
            FpowInstruction::Reset => "reset()void",
            FpowInstruction::ReloadALGO => "reload_algo()void",
            FpowInstruction::Deposit => "deposit(uint64,uint64)void",
            FpowInstruction::Withdraw => "withdraw(uint64)void",
            FpowInstruction::ClaimYield => "claim_yield(uint64)void",
            FpowInstruction::CompoundYield => "compound_yield()void",
            FpowInstruction::Buyback => "buyback()void",
            FpowInstruction::Bury => "bury(uint64)void",
            FpowInstruction::Wrap => "wrap(uint64)void",
            FpowInstruction::SetAdmin => "set_admin(address)void",
            FpowInstruction::NewVar => "new_var(uint64,byte[32],uint64)void",
            FpowInstruction::Liq => "liq()void",
        }
    }
}

/// Arguments for the Automate instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct AutomateArgs {
    pub amount: u64,
    pub deposit: u64,
    pub fee: u64,
    pub mask: u64,
    pub strategy: u8,
    pub reload: u64,
}

/// Arguments for the ClaimALGO instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ClaimALGOArgs {}

/// Arguments for the ClaimFPOW instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ClaimFPOWArgs {}

/// Arguments for the Deploy instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DeployArgs {
    pub amount: u64,
    pub squares: u32,
}

/// Arguments for the Log instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LogArgs {}

/// Arguments for the Reset instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ResetArgs {}

/// Arguments for the SetAdmin instruction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SetAdminArgs {
    pub admin: [u8; 32],
}

/// Arguments for the Wrap instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WrapArgs {
    pub amount: u64,
}

/// Arguments for the Buyback instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BuybackArgs {}

/// Arguments for the Bury instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BuryArgs {
    pub amount: u64,
}

/// Arguments for the ReloadALGO instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ReloadALGOArgs {}

/// Arguments for the Deposit instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct DepositArgs {
    pub amount: u64,
    pub compound_fee: u64,
}

/// Arguments for the Withdraw instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct WithdrawArgs {
    pub amount: u64,
}

/// Arguments for the ClaimYield instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct ClaimYieldArgs {
    pub amount: u64,
}

/// Arguments for the Checkpoint instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CheckpointArgs {}

/// Arguments for the Close instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CloseArgs {}

/// Arguments for the NewVar instruction
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewVarArgs {
    pub id: u64,
    pub commit: [u8; 32],
    pub samples: u64,
}

/// Arguments for the Liq instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LiqArgs {}

/// Arguments for the CompoundYield instruction
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct CompoundYieldArgs {}
