use bytemuck::{Pod, Zeroable};
use drillx_2::Solution;
use num_enum::TryFromPrimitive;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    consts::*,
    utils::{impl_instruction_from_bytes, impl_to_bytes},
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
#[rustfmt::skip]
pub enum OreInstruction {
    // User
    Claim = 0,
    Close = 1,
    Mine = 2,
    Open = 3,
    Reset = 4,
    Stake = 5,
    Update = 6,
    // Admin
    InitCoal = 100,
    InitWood = 101,
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
pub struct OpenArgs {
    pub bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MineArgs {
    pub digest: [u8; 16],
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct MineArgsV2 {
    pub digest: [u8; 16],
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

impl_to_bytes!(InitializeArgs);
impl_to_bytes!(OpenArgs);
impl_to_bytes!(MineArgs);
impl_to_bytes!(ClaimArgs);
impl_to_bytes!(StakeArgs);
impl_to_bytes!(UpgradeArgs);

impl_instruction_from_bytes!(InitializeArgs);
impl_instruction_from_bytes!(OpenArgs);
impl_instruction_from_bytes!(MineArgs);
impl_instruction_from_bytes!(ClaimArgs);
impl_instruction_from_bytes!(StakeArgs);
impl_instruction_from_bytes!(UpgradeArgs);

/// Builds an auth instruction.
pub fn auth(proof: Pubkey) -> Instruction {
    Instruction {
        program_id: NOOP_PROGRAM_ID,
        accounts: vec![],
        data: proof.to_bytes().to_vec(),
    }
}

/// Builds a claim instruction.
pub fn claim_coal(signer: Pubkey, beneficiary: Pubkey, amount: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[COAL_PROOF, signer.as_ref()], &crate::id()).0;
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &COAL_MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(beneficiary, false),
            AccountMeta::new(proof, false),
            AccountMeta::new_readonly(TREASURY_ADDRESS, false),
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

pub fn claim_wood(signer: Pubkey, beneficiary: Pubkey, amount: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[WOOD_PROOF, signer.as_ref()], &crate::id()).0;
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &WOOD_MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(beneficiary, false),
            AccountMeta::new(proof, false),
            AccountMeta::new_readonly(TREASURY_ADDRESS, false),
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

/// Builds a close instruction.
pub fn close_coal(signer: Pubkey) -> Instruction {
    let proof_pda = Pubkey::find_program_address(&[COAL_PROOF, signer.as_ref()], &crate::id());
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: OreInstruction::Close.to_vec(),
    }
}

pub fn close_wood(signer: Pubkey) -> Instruction {
    let proof_pda = Pubkey::find_program_address(&[WOOD_PROOF, signer.as_ref()], &crate::id());
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: OreInstruction::Close.to_vec(),
    }
}

/// Builds a mine instruction.
pub fn mine(
    signer: Pubkey,
    proof_authority: Pubkey,
    bus: Pubkey,
    solution: Solution,
) -> Instruction {
    let proof = Pubkey::find_program_address(&[COAL_PROOF, proof_authority.as_ref()], &crate::id()).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus, false),
            AccountMeta::new_readonly(COAL_CONFIG_ADDRESS, false),
            AccountMeta::new(proof, false),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Mine.to_vec(),
            MineArgs {
                digest: solution.d,
                nonce: solution.n,
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

pub fn chop_wood(
    signer: Pubkey,
    proof_authority: Pubkey,
    bus: Pubkey,
    solution: Solution,
) -> Instruction {
    let proof = Pubkey::find_program_address(&[WOOD_PROOF, proof_authority.as_ref()], &crate::id()).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus, false),
            AccountMeta::new_readonly(WOOD_CONFIG_ADDRESS, false),
            AccountMeta::new(proof, false),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Mine.to_vec(),
            MineArgs {
                digest: solution.d,
                nonce: solution.n,
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

/// Builds an open instruction.
pub fn open_coal(signer: Pubkey, miner: Pubkey, payer: Pubkey) -> Instruction {
    let proof_pda = Pubkey::find_program_address(&[COAL_PROOF, signer.as_ref()], &crate::id());
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(miner, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Open.to_vec(),
            OpenArgs { bump: proof_pda.1 }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}

pub fn open_wood(signer: Pubkey, miner: Pubkey, payer: Pubkey) -> Instruction {
    let proof_pda = Pubkey::find_program_address(&[WOOD_PROOF, signer.as_ref()], &crate::id());
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(miner, false),
            AccountMeta::new(payer, true),
            AccountMeta::new(proof_pda.0, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Open.to_vec(),
            OpenArgs { bump: proof_pda.1 }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}

/// Builds a reset instruction.
pub fn reset_coal(signer: Pubkey) -> Instruction {
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &COAL_MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(COAL_BUS_ADDRESSES[0], false),
            AccountMeta::new(COAL_BUS_ADDRESSES[1], false),
            AccountMeta::new(COAL_BUS_ADDRESSES[2], false),
            AccountMeta::new(COAL_BUS_ADDRESSES[3], false),
            AccountMeta::new(COAL_BUS_ADDRESSES[4], false),
            AccountMeta::new(COAL_BUS_ADDRESSES[5], false),
            AccountMeta::new(COAL_BUS_ADDRESSES[6], false),
            AccountMeta::new(COAL_BUS_ADDRESSES[7], false),
            AccountMeta::new(COAL_CONFIG_ADDRESS, false),
            AccountMeta::new(COAL_MINT_ADDRESS, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: OreInstruction::Reset.to_vec(),
    }
}

pub fn reset_wood(signer: Pubkey) -> Instruction {
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &WOOD_MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(WOOD_BUS_ADDRESSES[0], false),
            AccountMeta::new(WOOD_BUS_ADDRESSES[1], false),
            AccountMeta::new(WOOD_BUS_ADDRESSES[2], false),
            AccountMeta::new(WOOD_BUS_ADDRESSES[3], false),
            AccountMeta::new(WOOD_BUS_ADDRESSES[4], false),
            AccountMeta::new(WOOD_BUS_ADDRESSES[5], false),
            AccountMeta::new(WOOD_BUS_ADDRESSES[6], false),
            AccountMeta::new(WOOD_BUS_ADDRESSES[7], false),
            AccountMeta::new(WOOD_CONFIG_ADDRESS, false),
            AccountMeta::new(WOOD_MINT_ADDRESS, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: OreInstruction::Reset.to_vec(),
    }
}

/// Build a stake instruction.
pub fn stake_coal(signer: Pubkey, sender: Pubkey, amount: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[COAL_PROOF, signer.as_ref()], &crate::id()).0;
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &COAL_MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: [
            OreInstruction::Stake.to_vec(),
            StakeArgs {
                amount: amount.to_le_bytes(),
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

pub fn stake_wood(signer: Pubkey, sender: Pubkey, amount: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[WOOD_PROOF, signer.as_ref()], &crate::id()).0;
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &WOOD_MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: [
            OreInstruction::Stake.to_vec(),
            StakeArgs {
                amount: amount.to_le_bytes(),
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

// Build an update instruction.
pub fn update_coal(signer: Pubkey, miner: Pubkey) -> Instruction {
    let proof = Pubkey::find_program_address(&[COAL_PROOF, signer.as_ref()], &crate::id()).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(miner, false),
            AccountMeta::new(proof, false),
        ],
        data: OreInstruction::Update.to_vec(),
    }
}

pub fn update_wood(signer: Pubkey, miner: Pubkey) -> Instruction {
    let proof = Pubkey::find_program_address(&[WOOD_PROOF, signer.as_ref()], &crate::id()).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(miner, false),
            AccountMeta::new(proof, false),
        ],
        data: OreInstruction::Update.to_vec(),
    }
}

/// Builds an initialize instruction.
pub fn init_coal(signer: Pubkey) -> Instruction {
    let bus_pdas = [
        Pubkey::find_program_address(&[COAL_BUS, &[0]], &crate::id()),
        Pubkey::find_program_address(&[COAL_BUS, &[1]], &crate::id()),
        Pubkey::find_program_address(&[COAL_BUS, &[2]], &crate::id()),
        Pubkey::find_program_address(&[COAL_BUS, &[3]], &crate::id()),
        Pubkey::find_program_address(&[COAL_BUS, &[4]], &crate::id()),
        Pubkey::find_program_address(&[COAL_BUS, &[5]], &crate::id()),
        Pubkey::find_program_address(&[COAL_BUS, &[6]], &crate::id()),
        Pubkey::find_program_address(&[COAL_BUS, &[7]], &crate::id()),
    ];
    let config_pda = Pubkey::find_program_address(&[COAL_CONFIG], &crate::id());
    let mint_pda = Pubkey::find_program_address(&[COAL_MINT, MINT_NOISE.as_slice()], &crate::id());
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
            AccountMeta::new(config_pda.0, false),
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
            OreInstruction::InitCoal.to_vec(),
            InitializeArgs {
                bus_0_bump: bus_pdas[0].1,
                bus_1_bump: bus_pdas[1].1,
                bus_2_bump: bus_pdas[2].1,
                bus_3_bump: bus_pdas[3].1,
                bus_4_bump: bus_pdas[4].1,
                bus_5_bump: bus_pdas[5].1,
                bus_6_bump: bus_pdas[6].1,
                bus_7_bump: bus_pdas[7].1,
                config_bump: config_pda.1,
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

pub fn init_wood(signer: Pubkey) -> Instruction {
    let bus_pdas = [
        Pubkey::find_program_address(&[WOOD_BUS, &[0]], &crate::id()),
        Pubkey::find_program_address(&[WOOD_BUS, &[1]], &crate::id()),
        Pubkey::find_program_address(&[WOOD_BUS, &[2]], &crate::id()),
        Pubkey::find_program_address(&[WOOD_BUS, &[3]], &crate::id()),
        Pubkey::find_program_address(&[WOOD_BUS, &[4]], &crate::id()),
        Pubkey::find_program_address(&[WOOD_BUS, &[5]], &crate::id()),
        Pubkey::find_program_address(&[WOOD_BUS, &[6]], &crate::id()),
        Pubkey::find_program_address(&[WOOD_BUS, &[7]], &crate::id()),
    ];
    let config_pda = Pubkey::find_program_address(&[WOOD_CONFIG], &crate::id());
    let mint_pda = Pubkey::find_program_address(&[WOOD_MINT, MINT_NOISE.as_slice()], &crate::id());
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
            AccountMeta::new(config_pda.0, false),
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
            OreInstruction::InitCoal.to_vec(),
            InitializeArgs {
                bus_0_bump: bus_pdas[0].1,
                bus_1_bump: bus_pdas[1].1,
                bus_2_bump: bus_pdas[2].1,
                bus_3_bump: bus_pdas[3].1,
                bus_4_bump: bus_pdas[4].1,
                bus_5_bump: bus_pdas[5].1,
                bus_6_bump: bus_pdas[6].1,
                bus_7_bump: bus_pdas[7].1,
                config_bump: config_pda.1,
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
