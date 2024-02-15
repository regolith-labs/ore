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
    self, account_info::AccountInfo, declare_id, entrypoint::ProgramResult, log::sol_log,
    program_error::ProgramError, pubkey::Pubkey,
};

// TODO Test admin and difficulty adjustment functions
// TODO Increase decimals?

declare_id!("CeJShZEAzBLwtcLQvbZc7UT38e4nUTn63Za5UFyYYDTS");

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

/// Processes the incoming instruction
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    sol_log("1");
    let (tag, data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    sol_log(format!("2: {:?}", tag).as_str());

    let ix = OreInstruction::try_from(*tag).or(Err(ProgramError::InvalidInstructionData))?;
    sol_log(format!("3: {:?}", ix).as_str());
    match ix {
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
