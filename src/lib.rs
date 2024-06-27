pub mod consts;
pub mod error;
pub mod instruction;
pub mod loaders;
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

declare_id!("oreFHcE6FvJTrsfaYca4mVeZn7J7T6oZS9FAvW9eg4q");

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
        OreInstruction::Claim => process_claim(program_id, accounts, data)?,
        OreInstruction::Close => process_close(program_id, accounts, data)?,
        OreInstruction::Crown => process_crown(program_id, accounts, data)?,
        OreInstruction::Mine => process_mine(program_id, accounts, data)?,
        OreInstruction::Open => process_open(program_id, accounts, data)?,
        OreInstruction::Reset => process_reset(program_id, accounts, data)?,
        OreInstruction::Stake => process_stake(program_id, accounts, data)?,
        OreInstruction::Update => process_update(program_id, accounts, data)?,
        OreInstruction::Upgrade => process_upgrade(program_id, accounts, data)?,
        OreInstruction::Initialize => process_initialize(program_id, accounts, data)?,
    }

    Ok(())
}
