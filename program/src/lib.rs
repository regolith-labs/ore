mod close;
mod initialize;
mod log;
mod mine;
mod open;
mod reset;
mod set_admin;
mod set_fee_collector;
mod set_fee_rate;
mod swap;

use close::*;
use initialize::*;
use log::*;
use mine::*;
use open::*;
use reset::*;
use set_admin::*;
use set_fee_collector::*;
use set_fee_rate::*;
use swap::*;

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
        OreInstruction::Open => process_open(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Log => process_log(accounts, data)?,
        OreInstruction::Mine => process_mine(accounts, data)?,
        OreInstruction::Swap => process_swap(accounts, data)?,
        OreInstruction::Reset => process_reset(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,

        // Admin
        OreInstruction::SetAdmin => process_set_admin(accounts, data)?,
        OreInstruction::SetFeeCollector => process_set_fee_collector(accounts, data)?,
        OreInstruction::SetFeeRate => process_set_fee_rate(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
