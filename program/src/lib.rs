mod automate;
mod boost;
mod bury;
mod claim_ore;
mod claim_seeker;
mod claim_sol;
mod deploy;
mod initialize;
mod log;
mod migrate_miner;
mod migrate_squares;
mod reset;
mod set_admin;
mod set_fee_collector;
mod whitelist;
mod wrap;

use automate::*;
use boost::*;
use bury::*;
use claim_ore::*;
use claim_seeker::*;
use claim_sol::*;
use deploy::*;
use initialize::*;
use log::*;
use migrate_miner::*;
use migrate_squares::*;
use reset::*;
use set_admin::*;
use set_fee_collector::*;
use wrap::*;

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
        OreInstruction::Automate => process_automate(accounts, data)?,
        OreInstruction::Boost => process_boost(accounts, data)?,
        OreInstruction::ClaimSOL => process_claim_sol(accounts, data)?,
        OreInstruction::ClaimORE => process_claim_ore(accounts, data)?,
        OreInstruction::Deploy => process_deploy(accounts, data)?,
        OreInstruction::Log => process_log(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,
        OreInstruction::Reset => process_reset(accounts, data)?,

        // Admin
        OreInstruction::Bury => process_bury(accounts, data)?,
        OreInstruction::Wrap => process_wrap(accounts, data)?,
        OreInstruction::SetAdmin => process_set_admin(accounts, data)?,
        OreInstruction::SetFeeCollector => process_set_fee_collector(accounts, data)?,
        OreInstruction::MigrateMiner => process_migrate_miner(accounts, data)?,
        OreInstruction::MigrateSquares => process_migrate_squares(accounts, data)?,

        // Seeker
        OreInstruction::ClaimSeeker => process_claim_seeker(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
