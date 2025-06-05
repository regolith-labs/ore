mod buy;
mod close;
mod deposit;
mod free;
mod mine;
mod open;
mod sell;
mod withdraw;

use buy::*;
use close::*;
use deposit::*;
use free::*;
use mine::*;
use open::*;
use sell::*;
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
        OreInstruction::Free => process_free(accounts, data)?,

        // Trade
        OreInstruction::Buy => process_buy(accounts, data)?,
        OreInstruction::Sell => process_sell(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
