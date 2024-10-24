use shank::{ShankInstruction, ShankType};
use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ShankInstruction, TryFromPrimitive)]
#[rustfmt::skip]
pub enum OreInstruction {
    // User
    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, writable, name="beneficiary", desc="beneficiary")]
    #[account(2, writable, name="proof", desc="proof")]
    #[account(3, name="treasury", desc="treasury")]
    #[account(4, writable, name="treasury_tokens", desc="treasury_tokens")]
    #[account(5, name="token_program", desc="token_program")]
    Claim = 0,

    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, writable, name="proof", desc="proof")]
    #[account(2, name="system_program", desc="system_program")]
    Close = 1,

    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, writable, name="bus", desc="bus")]
    #[account(2, name="config", desc="config")]
    #[account(3, writable, name="proof", desc="proof")]
    #[account(4, name="instructions_sysvar", desc="instructions_sysvar")]
    #[account(5, name="slot_hashes_sysvar", desc="slot_hashes_sysvar")]
    Mine = 2,

    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, name="miner", desc="miner")]
    #[account(2, writable, name="payer", desc="payer")]
    #[account(3, writable, name="proof_pda", desc="proof_pda")]
    #[account(4, name="system_program", desc="system_program")]
    #[account(5, name="slot_hashes_sysvar", desc="slot_hashes_sysvar")]
    Open = 3,

    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, writable, name="bus_0", desc="bus_0")]
    #[account(2, writable, name="bus_1", desc="bus_1")]
    #[account(3, writable, name="bus_2", desc="bus_2")]
    #[account(4, writable, name="bus_3", desc="bus_3")]
    #[account(5, writable, name="bus_4", desc="bus_4")]
    #[account(6, writable, name="bus_5", desc="bus_5")]
    #[account(7, writable, name="bus_6", desc="bus_6")]
    #[account(8, writable, name="bus_7", desc="bus_7")]
    #[account(9, writable, name="config", desc="config")]
    #[account(10, writable, name="mint", desc="mint")]
    #[account(11, writable, name="treasury", desc="treasury")]
    #[account(12, writable, name="treasury_tokens", desc="treasury_tokens")]
    #[account(13, name="token_program", desc="token_program")]
    Reset = 4,

    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, writable, name="proof", desc="proof")]
    #[account(2, writable, name="sender", desc="sender")]
    #[account(3, writable, name="treasury_tokens", desc="treasury_tokens")]
    #[account(4, name="token_program", desc="token_program")]
    Stake = 5,

    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, name="miner", desc="miner")]
    #[account(2, writable, name="proof", desc="proof")]
    Update = 6,

    #[account(0, writable, signer, name="signer", desc="signer")]
    #[account(1, name="beneficiary", desc="beneficiary")]
    #[account(2, writable, name="mint", desc="mint")]
    #[account(3, writable, name="mint_v1", desc="mint_v1")]
    #[account(4, writable, name="sender", desc="sender")]
    #[account(5, writable, name="treasury", desc="treasury")]
    #[account(6, name="token_program", desc="token_program")]
    Upgrade = 7,

    // Admin
    #[account(0, signer, name="signer", desc="signer")]
    #[account(1, writable, name="bus_0", desc="bus_0")]
    #[account(2, writable, name="bus_1", desc="bus_1")]
    #[account(3, writable, name="bus_2", desc="bus_2")]
    #[account(4, writable, name="bus_3", desc="bus_3")]
    #[account(5, writable, name="bus_4", desc="bus_4")]
    #[account(6, writable, name="bus_5", desc="bus_5")]
    #[account(7, writable, name="bus_6", desc="bus_6")]
    #[account(8, writable, name="bus_7", desc="bus_7")]
    #[account(9, writable, name="config", desc="config")]
    #[account(10, writable, name="metadata", desc="metadata")]
    #[account(11, writable, name="mint", desc="mint")]
    #[account(12, writable, name="treasury", desc="treasury")]
    #[account(13, writable, name="treasury_tokens", desc="treasury_tokens")]
    #[account(14, name="system_program", desc="system_program")]
    #[account(15, name="token_program", desc="token_program")]
    #[account(16, name="associated_token_program", desc="associated_token_program")]
    #[account(17, name="metadata_program", desc="metadata_program")]
    #[account(18, name="rent_sysvar", desc="rent_sysvar")]
    Initialize = 100,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Claim {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Mine {
    pub digest: [u8; 16],
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Open {
    pub bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Reset {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Stake {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Update {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Upgrade {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, ShankType, Zeroable)]
pub struct Initialize {
    pub bus_0_bump: u8,
    pub bus_1_bump: u8,
    pub bus_2_bump: u8,
    pub bus_3_bump: u8,
    pub bus_4_bump: u8,
    pub bus_5_bump: u8,
    pub bus_6_bump: u8,
    pub bus_7_bump: u8,
    pub config_bump: u8,
    pub metadata_bump: u8,
    pub mint_bump: u8,
    pub treasury_bump: u8,
}

instruction!(OreInstruction, Claim);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Mine);
instruction!(OreInstruction, Open);
instruction!(OreInstruction, Reset);
instruction!(OreInstruction, Stake);
instruction!(OreInstruction, Update);
instruction!(OreInstruction, Upgrade);
instruction!(OreInstruction, Initialize);
