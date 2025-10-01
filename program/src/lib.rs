mod automate;
mod boost;
mod bury;
mod checkpoint;
mod claim_ore;
mod claim_seeker;
mod claim_sol;
mod claim_yield;
mod deploy;
mod deposit;
// mod initialize;
mod log;
mod reset;
mod set_admin;
mod set_fee_collector;
mod whitelist;
mod withdraw;
mod wrap;

use automate::*;
use boost::*;
use bury::*;
use checkpoint::*;
use claim_ore::*;
use claim_seeker::*;
use claim_sol::*;
use claim_yield::*;
use deploy::*;
use deposit::*;
// use initialize::*;
use log::*;
use reset::*;
use set_admin::*;
use set_fee_collector::*;
use whitelist::*;
use withdraw::*;
use wrap::*;

use ore_api::instruction::*;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    // let (ix, data) = parse_instruction(&ore_api::ID, program_id, data)?;

    // match ix {
    //     // Miner
    //     OreInstruction::Automate => process_automate(accounts, data)?,
    //     OreInstruction::Boost => process_boost(accounts, data)?,
    //     OreInstruction::ClaimSOL => process_claim_sol(accounts, data)?,
    //     OreInstruction::ClaimORE => process_claim_ore(accounts, data)?,
    //     OreInstruction::Deploy => process_deploy(accounts, data)?,
    //     OreInstruction::Log => process_log(accounts, data)?,
    //     OreInstruction::Initialize => process_initialize(accounts, data)?,
    //     OreInstruction::Reset => process_reset(accounts, data)?,

    //     // Staker
    //     OreInstruction::Deposit => process_deposit(accounts, data)?,
    //     OreInstruction::Withdraw => process_withdraw(accounts, data)?,
    //     OreInstruction::ClaimYield => process_claim_yield(accounts, data)?,

    //     // Admin
    //     OreInstruction::Bury => process_bury(accounts, data)?,
    //     OreInstruction::Wrap => process_wrap(accounts, data)?,
    //     OreInstruction::SetAdmin => process_set_admin(accounts, data)?,
    //     OreInstruction::SetFeeCollector => process_set_fee_collector(accounts, data)?,

    //     // Seeker
    //     OreInstruction::ClaimSeeker => process_claim_seeker(accounts, data)?,
    // }

    Ok(())
}

entrypoint!(process_instruction);
