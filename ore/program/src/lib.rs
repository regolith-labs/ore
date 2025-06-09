mod close;
mod commit;
mod decommit;
mod mine;
mod open;
mod swap;

use close::*;
use commit::*;
use decommit::*;
use mine::*;
use open::*;
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
        OreInstruction::Open => process_open(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Commit => process_commit(accounts, data)?,
        OreInstruction::Decommit => process_decommit(accounts, data)?,
        OreInstruction::Mine => process_mine(accounts, data)?,
        OreInstruction::Swap => process_swap(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
