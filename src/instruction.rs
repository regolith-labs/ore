use bytemuck::{Pod, Zeroable};
use num_enum::TryFromPrimitive;
use shank::ShankInstruction;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    impl_instruction_from_bytes, impl_to_bytes, state::Hash, BUS, METADATA, MINT, MINT_ADDRESS,
    MINT_NOISE, PROOF, TREASURY, TREASURY_ADDRESS,
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
    #[account(2, name = "proof", desc = "Ore proof account", writable)]
    #[account(3, name = "system_program", desc = "Solana system program")]
    Register = 1,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "bus", desc = "Ore bus account", writable)]
    #[account(3, name = "proof", desc = "Ore proof account", writable)]
    #[account(4, name = "treasury", desc = "Ore treasury account")]
    #[account(5, name = "slot_hashes", desc = "Solana slot hashes sysvar")]
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
    #[account(12, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(13, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(14, name = "system_program", desc = "Solana system program")]
    #[account(15, name = "token_program", desc = "SPL token program")]
    #[account(16, name = "associated_token_program", desc = "SPL associated token program")]
    #[account(17, name = "mpl_metadata_program", desc = "Metaplex metadata program")]
    #[account(18, name = "rent", desc = "Solana rent sysvar")]
    Initialize = 100,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Admin signer", signer)]
    #[account(2, name = "treasury", desc = "Ore treasury account")]
    UpdateAdmin = 101,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Admin signer", signer)]
    #[account(2, name = "treasury", desc = "Ore treasury account")]
    UpdateDifficulty = 102,
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
    pub hash: Hash,
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ClaimArgs {
    pub amount: [u8; 8],
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
impl_to_bytes!(RegisterArgs);
impl_to_bytes!(MineArgs);
impl_to_bytes!(ClaimArgs);
impl_to_bytes!(UpdateAdminArgs);
impl_to_bytes!(UpdateDifficultyArgs);

impl_instruction_from_bytes!(InitializeArgs);
impl_instruction_from_bytes!(RegisterArgs);
impl_instruction_from_bytes!(MineArgs);
impl_instruction_from_bytes!(ClaimArgs);
impl_instruction_from_bytes!(UpdateAdminArgs);
impl_instruction_from_bytes!(UpdateDifficultyArgs);

/// Builds a reset instruction.
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

/// Builds a register instruction.
pub fn register(signer: Pubkey) -> Instruction {
    let proof_pda = Pubkey::find_program_address(&[PROOF, signer.as_ref()], &crate::id());
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: [
            OreInstruction::Register.to_vec(),
            RegisterArgs { bump: proof_pda.1 }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}

/// Builds a mine instruction.
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

/// Builds a claim instruction.
pub fn claim(signer: Pubkey, beneficiary: Pubkey, amount: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[PROOF, signer.as_ref()], &crate::id()).0;
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(beneficiary, false),
            AccountMeta::new(proof, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: [
            OreInstruction::Claim.to_vec(),
            ClaimArgs {
                amount: amount.to_le_bytes(),
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

/// Builds an initialize instruction.
pub fn initialize(signer: Pubkey) -> Instruction {
    let bus_pdas = [
        Pubkey::find_program_address(&[BUS, &[0]], &crate::id()),
        Pubkey::find_program_address(&[BUS, &[1]], &crate::id()),
        Pubkey::find_program_address(&[BUS, &[2]], &crate::id()),
        Pubkey::find_program_address(&[BUS, &[3]], &crate::id()),
        Pubkey::find_program_address(&[BUS, &[4]], &crate::id()),
        Pubkey::find_program_address(&[BUS, &[5]], &crate::id()),
        Pubkey::find_program_address(&[BUS, &[6]], &crate::id()),
        Pubkey::find_program_address(&[BUS, &[7]], &crate::id()),
    ];
    let mint_pda = Pubkey::find_program_address(&[MINT, MINT_NOISE.as_slice()], &crate::id());
    let treasury_pda = Pubkey::find_program_address(&[TREASURY], &crate::id());
    let treasury_tokens =
        spl_associated_token_account::get_associated_token_address(&treasury_pda.0, &mint_pda.0);
    let metadata_pda = Pubkey::find_program_address(
        &[
            METADATA,
            mpl_token_metadata::ID.as_ref(),
            mint_pda.0.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus_pdas[0].0, false),
            AccountMeta::new(bus_pdas[1].0, false),
            AccountMeta::new(bus_pdas[2].0, false),
            AccountMeta::new(bus_pdas[3].0, false),
            AccountMeta::new(bus_pdas[4].0, false),
            AccountMeta::new(bus_pdas[5].0, false),
            AccountMeta::new(bus_pdas[6].0, false),
            AccountMeta::new(bus_pdas[7].0, false),
            AccountMeta::new(metadata_pda.0, false),
            AccountMeta::new(mint_pda.0, false),
            AccountMeta::new(treasury_pda.0, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            AccountMeta::new_readonly(mpl_token_metadata::ID, false),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
        ],
        data: [
            OreInstruction::Initialize.to_vec(),
            InitializeArgs {
                bus_0_bump: bus_pdas[0].1,
                bus_1_bump: bus_pdas[1].1,
                bus_2_bump: bus_pdas[2].1,
                bus_3_bump: bus_pdas[3].1,
                bus_4_bump: bus_pdas[4].1,
                bus_5_bump: bus_pdas[5].1,
                bus_6_bump: bus_pdas[6].1,
                bus_7_bump: bus_pdas[7].1,
                metadata_bump: metadata_pda.1,
                mint_bump: mint_pda.1,
                treasury_bump: treasury_pda.1,
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

/// Builds an update_admin instruction.
pub fn update_admin(signer: Pubkey, new_admin: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(TREASURY_ADDRESS, false),
        ],
        data: [
            OreInstruction::UpdateAdmin.to_vec(),
            UpdateAdminArgs { new_admin }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}

/// Builds an update_difficulty instruction.
pub fn update_difficulty(signer: Pubkey, new_difficulty: Hash) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(TREASURY_ADDRESS, false),
        ],
        data: [
            OreInstruction::UpdateDifficulty.to_vec(),
            UpdateDifficultyArgs { new_difficulty }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}
