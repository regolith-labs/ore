mod crank;
mod deposit;
mod payout;
mod withdraw;

use crank::*;
use deposit::*;
use payout::*;
use withdraw::*;

use ore_delegate_api::instruction::*;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&ore_delegate_api::ID, program_id, data)?;

    match ix {
        OreDelegateInstruction::Deposit => process_deposit(accounts, data)?,
        OreDelegateInstruction::Withdraw => process_withdraw(accounts, data)?,
        OreDelegateInstruction::Crank => process_crank(accounts, data)?,
        OreDelegateInstruction::Payout => process_payout(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
