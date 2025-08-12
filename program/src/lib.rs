mod claim;
mod close;
mod initialize;
mod mine;
mod open;
mod reset;
mod transfer_mint_authority;
mod update;

use claim::*;
use close::*;
use initialize::*;
use mine::*;
use open::*;
use reset::*;
use transfer_mint_authority::*;
use update::*;

use ore_api::instruction::*;
use steel::*;

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
        OreInstruction::Update => process_update(accounts, data)?,
        OreInstruction::Initialize => process_initialize(accounts, data)?,
        OreInstruction::TransferMintAuthority => process_transfer_mint_authority(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);
