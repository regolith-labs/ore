mod bury;
mod claim;
mod close;
mod deploy;
mod initialize;
mod payout;
mod reset;

use bury::*;
use claim::*;
use close::*;
use deploy::*;
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
        // User
        OreInstruction::Claim => process_claim(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Deploy => process_deploy(accounts, data)?,
        OreInstruction::Payout => process_payout(accounts, data)?,
        OreInstruction::Reset => process_reset(accounts, data)?,

        // Admin
        OreInstruction::Bury => process_bury(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
