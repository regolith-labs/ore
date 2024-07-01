mod claim;
mod close;
mod crown;
mod initialize;
mod mine;
mod open;
mod reset;
mod stake;
mod update;
mod upgrade;

use claim::*;
use close::*;
use crown::*;
use initialize::*;
use mine::*;
use open::*;
use reset::*;
use stake::*;
use update::*;
use upgrade::*;

use ore_api::instruction::*;
use solana_program::{
    self, account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

pub(crate) use utils;

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    if program_id.ne(&ore_api::id()) {
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
