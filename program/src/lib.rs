mod close;
mod deposit;
mod free;
mod mine;
mod open;
mod swap;
mod withdraw;

use close::*;
use deposit::*;
use free::*;
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
        // Mine
        OreInstruction::Open => process_open(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Mine => process_mine(accounts, data)?,

        // Stake
        OreInstruction::Deposit => process_deposit(accounts, data)?,
        OreInstruction::Withdraw => process_withdraw(accounts, data)?,

        // Trade
        OreInstruction::Free => process_free(accounts, data)?,
        OreInstruction::Swap => process_swap(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
