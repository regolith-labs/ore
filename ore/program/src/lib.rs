mod close;
mod commit;
mod decommit;
mod deposit;
mod mine;
mod open;
mod swap;
mod withdraw;

use close::*;
use commit::*;
use decommit::*;
use deposit::*;
use mine::*;
use open::*;
use swap::*;
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
        OreInstruction::Decommit => process_decommit(accounts, data)?,
        OreInstruction::Deposit => process_deposit(accounts, data)?,
        OreInstruction::Mine => process_mine(accounts, data)?,
        OreInstruction::Swap => process_swap(accounts, data)?,
        OreInstruction::Withdraw => process_withdraw(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
