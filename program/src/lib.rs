mod automate;
mod bury;
mod buyback;
mod checkpoint;
mod claim_fpow;
mod claim_algo;
mod claim_yield;
mod close;
mod compound_yield;
mod deploy;
mod deposit;
mod liq;
mod log;
mod new_var;
mod reload_algo;
mod reset;
mod set_admin;
mod withdraw;
mod wrap;

use automate::*;
use bury::*;
use buyback::*;
use checkpoint::*;
use claim_fpow::*;
use claim_algo::*;
use claim_yield::*;
use close::*;
use compound_yield::*;
use deploy::*;
use deposit::*;
use liq::*;
use log::*;
use new_var::*;
use reload_algo::*;
use reset::*;
use set_admin::*;
use withdraw::*;
use wrap::*;

use fpow_api::instruction::*;
use solana_security_txt::security_txt;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&fpow_api::ID, program_id, data)?;

    match ix {
        // Miner
        FpowInstruction::Automate => process_automate(accounts, data)?,
        FpowInstruction::Checkpoint => process_checkpoint(accounts, data)?,
        FpowInstruction::ClaimALGO => process_claim_algo(accounts, data)?,
        FpowInstruction::ClaimFPOW => process_claim_fpow(accounts, data)?,
        FpowInstruction::Deploy => process_deploy(accounts, data)?,
        FpowInstruction::Log => process_log(accounts, data)?,
        FpowInstruction::Close => process_close(accounts, data)?,
        FpowInstruction::Reset => process_reset(accounts, data)?,
        FpowInstruction::ReloadALGO => process_reload_algo(accounts, data)?,

        // Staker
        FpowInstruction::Deposit => process_deposit(accounts, data)?,
        FpowInstruction::Withdraw => process_withdraw(accounts, data)?,
        FpowInstruction::ClaimYield => process_claim_yield(accounts, data)?,
        FpowInstruction::CompoundYield => process_compound_yield(accounts, data)?,

        // Admin
        FpowInstruction::Buyback => process_buyback(accounts, data)?,
        FpowInstruction::Bury => process_bury(accounts, data)?,
        FpowInstruction::Wrap => process_wrap(accounts, data)?,
        FpowInstruction::SetAdmin => process_set_admin(accounts, data)?,
        FpowInstruction::NewVar => process_new_var(accounts, data)?,
        FpowInstruction::Liq => process_liq(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);

security_txt! {
    name: "fPOW",
    project_url: "https://fpow.supply",
    contacts: "email:support@fry.foundation",
    policy: "https://github.com/Fry-Foundation/fPOW/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/Fry-Foundation/fPOW"
}
