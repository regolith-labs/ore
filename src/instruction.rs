use bytemuck::{Pod, Zeroable};
use num_enum::TryFromPrimitive;
use shank::ShankInstruction;
use solana_program::pubkey::Pubkey;

use crate::{impl_instruction_from_bytes, impl_to_bytes};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ShankInstruction, TryFromPrimitive)]
#[rustfmt::skip]
pub enum OreInstruction {
    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "bus_0", desc = "Ore bus account 0", writable)]
    #[account(3, name = "bus_1", desc = "Ore bus account 1", writable)]
    #[account(4, name = "bus_2", desc = "Ore bus account 2", writable)]
    #[account(5, name = "bus_3", desc = "Ore bus account 3", writable)]
    #[account(6, name = "bus_4", desc = "Ore bus account 4", writable)]
    #[account(7, name = "bus_5", desc = "Ore bus account 5", writable)]
    #[account(8, name = "bus_6", desc = "Ore bus account 6", writable)]
    #[account(9, name = "bus_7", desc = "Ore bus account 7", writable)]
    #[account(10, name = "config", desc = "Ore config account")]
    #[account(11, name = "mint", desc = "Ore token mint account", writable)]
    #[account(12, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(13, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(14, name = "token_program", desc = "SPL token program")]
    Reset = 0,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "proof", desc = "Ore proof account", writable)]
    #[account(3, name = "system_program", desc = "Solana system program")]
    Register = 1,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "bus", desc = "Ore bus account", writable)]
    #[account(3, name = "config", desc = "Ore config account")]
    #[account(4, name = "noise", desc = "Ore noise account")]
    #[account(5, name = "proof", desc = "Ore proof account", writable)]
    #[account(6, name = "slot_hashes", desc = "Solana slot hashes sysvar")]
    Mine = 2,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "beneficiary", desc = "Beneficiary token account", writable)]
    #[account(3, name = "proof", desc = "Ore proof account", writable)]
    #[account(4, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(5, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(6, name = "token_program", desc = "SPL token program")]
    Claim = 3,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "proof", desc = "Ore proof account", writable)]
    #[account(3, name = "sender", desc = "Signer token account", writable)]
    #[account(4, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(5, name = "token_program", desc = "SPL token program")]
    Stake = 4,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "beneficiary", desc = "Beneficiary token account", writable)]
    #[account(3, name = "sender", desc = "Signer token account", writable)]
    #[account(4, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(5, name = "mint", desc = "Ore token mint account", writable)]
    #[account(6, name = "mint_v1", desc = "Ore v1 token mint account", writable)]
    #[account(7, name = "token_program", desc = "SPL token program")]
    Upgrade = 5,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "proof", desc = "Ore proof account", writable)]
    #[account(3, name = "system_program", desc = "Solana system program")]
    Deregister = 6,
    
    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Admin signer", signer)]
    #[account(2, name = "bus_0", desc = "Ore bus account 0", writable)]
    #[account(3, name = "bus_1", desc = "Ore bus account 1", writable)]
    #[account(4, name = "bus_2", desc = "Ore bus account 2", writable)]
    #[account(5, name = "bus_3", desc = "Ore bus account 3", writable)]
    #[account(6, name = "bus_4", desc = "Ore bus account 4", writable)]
    #[account(7, name = "bus_5", desc = "Ore bus account 5", writable)]
    #[account(8, name = "bus_6", desc = "Ore bus account 6", writable)]
    #[account(9, name = "bus_7", desc = "Ore bus account 7", writable)]
    #[account(10, name = "metadata", desc = "Ore mint metadata account", writable)]
    #[account(11, name = "mint", desc = "Ore mint account", writable)]
    #[account(12, name = "noise", desc = "Ore noise account", writable)]
    #[account(13, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(14, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(15, name = "system_program", desc = "Solana system program")]
    #[account(16, name = "token_program", desc = "SPL token program")]
    #[account(17, name = "associated_token_program", desc = "SPL associated token program")]
    #[account(18, name = "mpl_metadata_program", desc = "Metaplex metadata program")]
    #[account(19, name = "rent", desc = "Solana rent sysvar")]
    Initialize = 100,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Admin signer", signer)]
    #[account(2, name = "config", desc = "Ore config account", writable)]
    UpdateAdmin = 101,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Admin signer", signer)]
    #[account(2, name = "config", desc = "Ore config account", writable)]
    UpdateTolerance = 102,
    
    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Admin signer", signer)]
    #[account(2, name = "config", desc = "Ore config account", writable)]
    Pause = 103,
}

impl OreInstruction {
    pub fn to_vec(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializeArgs {
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

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct RegisterArgs {
    pub bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MineArgs {
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimArgs {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct StakeArgs {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UpgradeArgs {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UpdateAdminArgs {
    pub new_admin: Pubkey,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UpdateToleranceArgs {
    pub tolerance_liveness: u64,
    pub tolerance_spam: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct PauseArgs {
    pub paused: u8,
}

impl_to_bytes!(InitializeArgs);
impl_to_bytes!(RegisterArgs);
impl_to_bytes!(MineArgs);
impl_to_bytes!(ClaimArgs);
impl_to_bytes!(StakeArgs);
impl_to_bytes!(UpgradeArgs);
impl_to_bytes!(UpdateAdminArgs);
impl_to_bytes!(UpdateToleranceArgs);
impl_to_bytes!(PauseArgs);

impl_instruction_from_bytes!(InitializeArgs);
impl_instruction_from_bytes!(RegisterArgs);
impl_instruction_from_bytes!(MineArgs);
impl_instruction_from_bytes!(ClaimArgs);
impl_instruction_from_bytes!(StakeArgs);
impl_instruction_from_bytes!(UpgradeArgs);
impl_instruction_from_bytes!(UpdateAdminArgs);
impl_instruction_from_bytes!(UpdateToleranceArgs);
impl_instruction_from_bytes!(PauseArgs);
