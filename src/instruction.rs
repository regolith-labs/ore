use bytemuck::{Pod, Zeroable};
use num_enum::TryFromPrimitive;
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    sysvar,
};

use crate::{
    impl_instruction_from_bytes, impl_to_bytes, state::Hash, BUS, MINT_ADDRESS, PROOF,
    TREASURY_ADDRESS,
};

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
    #[account(10, name = "mint", desc = "Ore token mint account", writable)]
    #[account(11, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(12, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(13, name = "token_program", desc = "SPL token program")]
    Reset = 0,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "proof", desc = "Ore miner proof account", writable)]
    #[account(3, name = "system_program", desc = "Solana system program")]
    CreateProof = 1,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "bus", desc = "Ore bus account", writable)]
    #[account(3, name = "proof", desc = "Ore miner proof account", writable)]
    #[account(4, name = "treasury", desc = "Ore treasury account")]
    #[account(5, name = "slot_hashes", desc = "Solana slot hashes sysvar")]
    Mine = 2,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "beneficiary", desc = "Beneficiary token account", writable)]
    #[account(3, name = "mint", desc = "Ore token mint account")]
    #[account(4, name = "proof", desc = "Ore miner proof account", writable)]
    #[account(5, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(6, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(7, name = "token_program", desc = "SPL token program")]
    Claim = 3,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "admin", desc = "Admin signer", signer)]
    #[account(2, name = "bus_0", desc = "Ore bus account 0", writable)]
    #[account(3, name = "bus_1", desc = "Ore bus account 1", writable)]
    #[account(4, name = "bus_2", desc = "Ore bus account 2", writable)]
    #[account(5, name = "bus_3", desc = "Ore bus account 3", writable)]
    #[account(6, name = "bus_4", desc = "Ore bus account 4", writable)]
    #[account(7, name = "bus_5", desc = "Ore bus account 5", writable)]
    #[account(8, name = "bus_6", desc = "Ore bus account 6", writable)]
    #[account(9, name = "bus_7", desc = "Ore bus account 7", writable)]
    #[account(10, name = "mint", desc = "Ore token mint account")]
    #[account(11, name = "treasury", desc = "Ore treasury account")]
    #[account(12, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(13, name = "system_program", desc = "Solana system program")]
    #[account(14, name = "token_program", desc = "SPL token program")]
    #[account(15, name = "associated_token_program", desc = "SPL associated token program")]
    #[account(16, name = "rent", desc = "Solana rent sysvar")]
    Initialize = 100,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "treasury", desc = "Ore treasury account")]
    UpdateAdmin = 102,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "treasury", desc = "Ore treasury account")]
    UpdateDifficulty = 103,
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
    pub mint_bump: u8,
    pub treasury_bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct CreateProofArgs {
    pub bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MineArgs {
    pub hash: Hash,
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimArgs {
    pub amount: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UpdateAdminArgs {
    pub new_admin: Pubkey,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct UpdateDifficultyArgs {
    pub new_difficulty: Hash,
}

impl_to_bytes!(InitializeArgs);
impl_to_bytes!(CreateProofArgs);
impl_to_bytes!(MineArgs);
impl_to_bytes!(ClaimArgs);
impl_to_bytes!(UpdateAdminArgs);
impl_to_bytes!(UpdateDifficultyArgs);

impl_instruction_from_bytes!(InitializeArgs);
impl_instruction_from_bytes!(CreateProofArgs);
impl_instruction_from_bytes!(MineArgs);
impl_instruction_from_bytes!(ClaimArgs);
impl_instruction_from_bytes!(UpdateAdminArgs);
impl_instruction_from_bytes!(UpdateDifficultyArgs);

pub fn claim(signer: Pubkey, beneficiary: Pubkey, amount: u64) -> Instruction {
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );
    let proof = Pubkey::find_program_address(&[PROOF, signer.as_ref()], &crate::id()).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(beneficiary, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(proof, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: [
            OreInstruction::Claim.to_vec(),
            ClaimArgs { amount }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}

pub fn mine(signer: Pubkey, bus: Pubkey, hash: Hash, nonce: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[PROOF, signer.as_ref()], &crate::id()).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus, false),
            AccountMeta::new(proof, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Mine.to_vec(),
            MineArgs {
                hash,
                nonce: nonce.to_le_bytes(),
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

pub fn reset(signer: Pubkey) -> Instruction {
    let bus_0 = Pubkey::find_program_address(&[BUS, &[0]], &crate::id()).0;
    let bus_1 = Pubkey::find_program_address(&[BUS, &[1]], &crate::id()).0;
    let bus_2 = Pubkey::find_program_address(&[BUS, &[2]], &crate::id()).0;
    let bus_3 = Pubkey::find_program_address(&[BUS, &[3]], &crate::id()).0;
    let bus_4 = Pubkey::find_program_address(&[BUS, &[4]], &crate::id()).0;
    let bus_5 = Pubkey::find_program_address(&[BUS, &[5]], &crate::id()).0;
    let bus_6 = Pubkey::find_program_address(&[BUS, &[6]], &crate::id()).0;
    let bus_7 = Pubkey::find_program_address(&[BUS, &[7]], &crate::id()).0;
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus_0, false),
            AccountMeta::new(bus_1, false),
            AccountMeta::new(bus_2, false),
            AccountMeta::new(bus_3, false),
            AccountMeta::new(bus_4, false),
            AccountMeta::new(bus_5, false),
            AccountMeta::new(bus_6, false),
            AccountMeta::new(bus_7, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: OreInstruction::Reset.to_vec(),
    }
}
