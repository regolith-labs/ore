mod claim;
mod claim_seeker;
mod initialize;
mod initialize_square;
mod prospect;
mod reset;
mod set_admin;
mod set_fee_collector;
mod whitelist;

use claim::*;
use claim_seeker::*;
use initialize::*;
use initialize_square::*;
use prospect::*;
use reset::*;
use set_admin::*;
use set_fee_collector::*;

use ore_api::instruction::*;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&ore_api::ID, program_id, data)?;

    match ix {
        // User
        OreInstruction::Claim => process_claim(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,
        OreInstruction::InitializeSquare => process_initialize_square(accounts, data)?,
        OreInstruction::Prospect => process_prospect(accounts, data)?,
        OreInstruction::Reset => process_reset(accounts, data)?,

        // Admin
        OreInstruction::SetAdmin => process_set_admin(accounts, data)?,
        OreInstruction::SetFeeCollector => process_set_fee_collector(accounts, data)?,

        // Seeker
        OreInstruction::ClaimSeeker => process_claim_seeker(accounts, data)?,

        _ => return Err(ProgramError::InvalidInstructionData),
    }

    Ok(())
}

entrypoint!(process_instruction);
