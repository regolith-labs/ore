use bytemuck::{Pod, Zeroable};
use drillx::Solution;
use num_enum::TryFromPrimitive;
use ore_utils::*;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};

use crate::{
    consts::*,
    state::{bus_pda, config_pda, proof_pda, treasury_pda},
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // User
    Claim = 0,
    Close = 1,
    Mine = 2,
    Open = 3,
    Reset = 4,
    Stake = 5,
    Update = 6,
    Upgrade = 7,

    // Admin
    Initialize = 100,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Claim {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Mine {
    pub digest: [u8; 16],
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Open {
    pub bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Reset {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Stake {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Update {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Upgrade {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
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

/// Builds an auth instruction.
pub fn auth(proof: Pubkey) -> Instruction {
    Instruction {
        program_id: NOOP_PROGRAM_ID,
        accounts: vec![],
        data: proof.to_bytes().to_vec(),
    }
}

/// Builds a claim instruction.
pub fn claim(signer: Pubkey, beneficiary: Pubkey, amount: u64) -> Instruction {
    let proof = proof_pda(signer).0;
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
        data: Claim {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

/// Builds a close instruction.
pub fn close(signer: Pubkey) -> Instruction {
    let proof = proof_pda(signer).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(proof, false),
            AccountMeta::new_readonly(solana_program::system_program::id(), false),
        ],
        data: Close {}.to_bytes(),
    }
}

/// Builds a mine instruction.
pub fn mine(signer: Pubkey, authority: Pubkey, bus: Pubkey, solution: Solution) -> Instruction {
    let proof = proof_pda(authority).0;
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
        data: Mine {
            digest: solution.d,
            nonce: solution.n,
        }
        .to_bytes(),
    }
}

/// Builds an open instruction.
pub fn open(signer: Pubkey, miner: Pubkey, payer: Pubkey) -> Instruction {
    let proof_pda = proof_pda(signer);
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
        data: Open { bump: proof_pda.1 }.to_bytes(),
    }
}

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
        data: Reset {}.to_bytes(),
    }
}

/// Build a stake instruction.
pub fn stake(signer: Pubkey, sender: Pubkey, amount: u64) -> Instruction {
    let proof = proof_pda(signer).0;
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
        data: Stake {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

// Build an update instruction.
pub fn update(signer: Pubkey, miner: Pubkey) -> Instruction {
    let proof = proof_pda(signer).0;
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new_readonly(miner, false),
            AccountMeta::new(proof, false),
        ],
        data: Update {}.to_bytes(),
    }
}

// Build an upgrade instruction.
pub fn upgrade(signer: Pubkey, beneficiary: Pubkey, sender: Pubkey, amount: u64) -> Instruction {
    Instruction {
        program_id: crate::id(),
        accounts: vec![
            AccountMeta::new(signer, true),
            AccountMeta::new(beneficiary, false),
            AccountMeta::new(MINT_ADDRESS, false),
            AccountMeta::new(MINT_V1_ADDRESS, false),
            AccountMeta::new(sender, false),
            AccountMeta::new(TREASURY_ADDRESS, false),
            AccountMeta::new_readonly(spl_token::id(), false),
        ],
        data: Upgrade {
            amount: amount.to_le_bytes(),
        }
        .to_bytes(),
    }
}

/// Builds an initialize instruction.
pub fn initialize(signer: Pubkey) -> Instruction {
    let bus_pdas = [
        bus_pda(0),
        bus_pda(1),
        bus_pda(2),
        bus_pda(3),
        bus_pda(4),
        bus_pda(5),
        bus_pda(6),
        bus_pda(7),
    ];
    let config_pda = config_pda();
    let mint_pda = Pubkey::find_program_address(&[MINT, MINT_NOISE.as_slice()], &crate::id());
    let treasury_pda = treasury_pda();
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
        data: Initialize {
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
        .to_bytes(),
    }
}
