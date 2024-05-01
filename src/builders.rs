use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    instruction::{
        ClaimArgs, InitializeArgs, MineArgs, OreInstruction, PauseArgs, RegisterArgs, StakeArgs,
        UpdateAdminArgs, UpdateToleranceArgs,
    },
    BUS, BUS_ADDRESSES, CONFIG, CONFIG_ADDRESS, METADATA, MINT, MINT_ADDRESS, MINT_NOISE, PROOF,
    TREASURY, TREASURY_ADDRESS,
};

/// Builds a reset instruction.
pub fn reset(signer: Pubkey) -> Instruction {
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
    );
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(BUS_ADDRESSES[0], false),
            AccountMeta::new(BUS_ADDRESSES[1], false),
            AccountMeta::new(BUS_ADDRESSES[2], false),
            AccountMeta::new(BUS_ADDRESSES[3], false),
            AccountMeta::new(BUS_ADDRESSES[4], false),
            AccountMeta::new(BUS_ADDRESSES[5], false),
            AccountMeta::new(BUS_ADDRESSES[6], false),
            AccountMeta::new(BUS_ADDRESSES[7], false),
            AccountMeta::new(CONFIG_ADDRESS, false),
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
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Register.to_vec(),
            RegisterArgs { bump: proof_pda.1 }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}

/// Builds a mine instruction.
pub fn mine(signer: Pubkey, bus: Pubkey, nonce: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[PROOF, signer.as_ref()], &crate::id()).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(bus, false),
            AccountMeta::new_readonly(CONFIG_ADDRESS, false),
            AccountMeta::new(proof, false),
            AccountMeta::new_readonly(sysvar::instructions::id(), false),
            AccountMeta::new_readonly(sysvar::slot_hashes::id(), false),
        ],
        data: [
            OreInstruction::Mine.to_vec(),
            MineArgs {
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

/// Build a stake instruction.
pub fn stake(signer: Pubkey, sender: Pubkey, amount: u64) -> Instruction {
    let proof = Pubkey::find_program_address(&[PROOF, signer.as_ref()], &crate::id()).0;
    let treasury_tokens = spl_associated_token_account::get_associated_token_address(
        &TREASURY_ADDRESS,
        &MINT_ADDRESS,
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
    let config_pda = Pubkey::find_program_address(&[CONFIG], &crate::id());
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
            AccountMeta::new(config_pda.0, false),
            AccountMeta::new(metadata_pda.0, false),
            AccountMeta::new(mint_pda.0, false),
            AccountMeta::new(treasury_pda.0, false),
            AccountMeta::new(treasury_tokens, false),
            AccountMeta::new_readonly(system_program::id(), false),
            AccountMeta::new_readonly(spl_token::id(), false),
            AccountMeta::new_readonly(spl_associated_token_account::id(), false),
            // AccountMeta::new_readonly(mpl_token_metadata::ID, false),
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

/// Build an update_admin instruction.
pub fn update_admin(signer: Pubkey, new_admin: Pubkey) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(CONFIG_ADDRESS, false),
        ],
        data: [
            OreInstruction::UpdateAdmin.to_vec(),
            UpdateAdminArgs { new_admin }.to_bytes().to_vec(),
        ]
        .concat(),
    }
}

/// Build an update_tolerance instruction.
pub fn update_tolerance(
    signer: Pubkey,
    new_liveness_tolerance: u64,
    new_spam_tolerance: u64,
) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(CONFIG_ADDRESS, false),
        ],
        data: [
            OreInstruction::UpdateTolerance.to_vec(),
            UpdateToleranceArgs {
                tolerance_liveness: new_liveness_tolerance,
                tolerance_spam: new_spam_tolerance,
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}

/// Build a pause instruction.
pub fn pause(signer: Pubkey, paused: bool) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(CONFIG_ADDRESS, false),
        ],
        data: [
            OreInstruction::UpdateAdmin.to_vec(),
            PauseArgs {
                paused: paused as u8,
            }
            .to_bytes()
            .to_vec(),
        ]
        .concat(),
    }
}
