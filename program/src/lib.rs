mod claim;
mod close;
mod initialize;
mod mine;
mod open;
mod reset;
mod stake;
mod update;
mod upgrade;

use claim::*;
use close::*;
use initialize::*;
use mine::*;
use open::*;
use reset::*;
use stake::*;
use update::*;
use upgrade::*;

use ore_api::instruction::*;
use solana_include_idl::{include_idl, parse::IdlType};
use steel::*;

// Include the compressed IDL in an ELF section on the program binary.
include_idl!(IdlType::Codama, concat!(env!("OUT_DIR"), "/codama.idl.zip"));
#[allow(deprecated)]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&ore_api::ID, program_id, data)?;

    match ix {
        OreInstruction::Claim => process_claim(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Mine => process_mine(accounts, data)?,
        OreInstruction::Open => process_open(accounts, data)?,
        OreInstruction::Reset => process_reset(accounts, data)?,
        OreInstruction::Stake => process_stake(accounts, data)?,
        OreInstruction::Update => process_update(accounts, data)?,
        OreInstruction::Upgrade => process_upgrade(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
