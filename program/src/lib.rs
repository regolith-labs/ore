mod boost;
mod claim_ore;
mod claim_seeker;
mod claim_sol;
mod initialize;
mod log;
mod prospect;
mod redeem;
mod reset;
mod set_admin;
mod set_fee_collector;
mod whitelist;

use boost::*;
use claim_ore::*;
use claim_seeker::*;
use claim_sol::*;
use initialize::*;
use log::*;
use prospect::*;
use redeem::*;
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
        OreInstruction::Boost => process_boost(accounts, data)?,
        OreInstruction::ClaimSOL => process_claim_sol(accounts, data)?,
        OreInstruction::ClaimORE => process_claim_ore(accounts, data)?,
        OreInstruction::Log => process_log(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,
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
