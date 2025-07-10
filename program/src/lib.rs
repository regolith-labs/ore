mod close;
mod commit;
mod deposit;
mod log;
mod mine;
mod open;
mod set_admin;
mod set_block_limit;
mod set_fee_collector;
mod set_fee_rate;
mod swap;
mod uncommit;
mod withdraw;

use close::*;
use commit::*;
use deposit::*;
use log::*;
use mine::*;
use open::*;
use set_admin::*;
use set_block_limit::*;
use set_fee_collector::*;
use set_fee_rate::*;
use swap::*;
use uncommit::*;
use withdraw::*;

use ore_api::instruction::*;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&ore_api::ID, program_id, data)?;

    match ix {
        OreInstruction::Open => process_open(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Commit => process_commit(accounts, data)?,
        OreInstruction::Deposit => process_deposit(accounts, data)?,
        OreInstruction::Log => process_log(accounts, data)?,
        OreInstruction::Mine => process_mine(accounts, data)?,
        OreInstruction::Swap => process_swap(accounts, data)?,
        OreInstruction::Uncommit => process_uncommit(accounts, data)?,
        OreInstruction::Withdraw => process_withdraw(accounts, data)?,
        OreInstruction::SetAdmin => process_set_admin(accounts, data)?,
        OreInstruction::SetBlockLimit => process_set_block_limit(accounts, data)?,
        OreInstruction::SetFeeCollector => process_set_fee_collector(accounts, data)?,
        OreInstruction::SetFeeRate => process_set_fee_rate(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
