pub mod consts;
pub mod error;
pub mod instruction;
mod loaders;
mod processor;
pub mod state;
pub mod utils;

pub use consts::*;
use instruction::*;
use processor::*;
use solana_program::{
    self, account_info::AccountInfo, declare_id, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey,
};

// TODO Set START_AT before launch.
// TODO Set pubkey consts for derived mainnet pdas before lanch.

declare_id!("oreoDL2qcXyBdaYTEfd7F5MFLY5PAqqDooPLZv1XdBP");

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if program_id.ne(&crate::id()) {
        return Err(ProgramError::IncorrectProgramId);
    }

    let (tag, data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    match OreInstruction::try_from(*tag).or(Err(ProgramError::InvalidInstructionData))? {
        OreInstruction::Reset => process_reset(program_id, accounts, data)?,
        OreInstruction::Register => process_register(program_id, accounts, data)?,
        OreInstruction::Mine => process_mine(program_id, accounts, data)?,
        OreInstruction::Claim => process_claim(program_id, accounts, data)?,
        OreInstruction::Initialize => process_initialize(program_id, accounts, data)?,
        OreInstruction::UpdateAdmin => process_update_admin(program_id, accounts, data)?,
        OreInstruction::UpdateDifficulty => process_update_difficulty(program_id, accounts, data)?,
    }

    Ok(())
}
