mod bet;
mod close;
mod initialize;
mod payout;
mod reset;

use bet::*;
use close::*;
use initialize::*;
use payout::*;
use reset::*;

use ore_api::instruction::*;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&ore_api::ID, program_id, data)?;

    match ix {
        OreInstruction::Bet => process_bet(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Reset => process_reset(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,
        OreInstruction::Payout => process_payout(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
