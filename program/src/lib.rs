mod automate;
mod bury;
mod buyback;
mod checkpoint;
mod claim_ore;
mod claim_sol;
mod claim_yield;
mod close;
mod compound_yield;
mod deploy;
mod deposit;
mod liq;
mod log;
mod migrate_miner;
mod new_var;
mod reload_sol;
mod reset;
mod set_admin;
mod set_admin_fee;
mod set_fee_collector;
mod set_swap_program;
mod set_var_address;
mod withdraw;
mod wrap;

use automate::*;
use bury::*;
use buyback::*;
use checkpoint::*;
use claim_ore::*;
use claim_sol::*;
use claim_yield::*;
use close::*;
use compound_yield::*;
use deploy::*;
use deposit::*;
use liq::*;
use log::*;
use migrate_miner::*;
use new_var::*;
use reload_sol::*;
use reset::*;
use set_admin::*;
use set_admin_fee::*;
use set_fee_collector::*;
use set_swap_program::*;
use set_var_address::*;
use withdraw::*;
use wrap::*;

use ore_api::instruction::*;
use solana_security_txt::security_txt;
use steel::*;

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (ix, data) = parse_instruction(&ore_api::ID, program_id, data)?;

    match ix {
        // Miner
        OreInstruction::Automate => process_automate(accounts, data)?,
        OreInstruction::Checkpoint => process_checkpoint(accounts, data)?,
        OreInstruction::ClaimSOL => process_claim_sol(accounts, data)?,
        OreInstruction::ClaimORE => process_claim_ore(accounts, data)?,
        OreInstruction::Deploy => process_deploy(accounts, data)?,
        OreInstruction::Log => process_log(accounts, data)?,
        OreInstruction::Close => process_close(accounts, data)?,
        OreInstruction::Reset => process_reset(accounts, data)?,
        OreInstruction::ReloadSOL => process_reload_sol(accounts, data)?,

        // Staker
        OreInstruction::Deposit => process_deposit(accounts, data)?,
        OreInstruction::Withdraw => process_withdraw(accounts, data)?,
        OreInstruction::ClaimYield => process_claim_yield(accounts, data)?,
        OreInstruction::CompoundYield => process_compound_yield(accounts, data)?,

        // Admin
        OreInstruction::Buyback => process_buyback(accounts, data)?,
        OreInstruction::Bury => process_bury(accounts, data)?,
        OreInstruction::Wrap => process_wrap(accounts, data)?,
        OreInstruction::SetAdmin => process_set_admin(accounts, data)?,
        OreInstruction::SetFeeCollector => process_set_fee_collector(accounts, data)?,
        OreInstruction::SetSwapProgram => process_set_swap_program(accounts, data)?,
        OreInstruction::SetVarAddress => process_set_var_address(accounts, data)?,
        OreInstruction::NewVar => process_new_var(accounts, data)?,
        OreInstruction::SetAdminFee => process_set_admin_fee(accounts, data)?,
        OreInstruction::Liq => process_liq(accounts, data)?,
        OreInstruction::MigrateMiner => process_migrate_miner(accounts, data)?,
    }

    Ok(())
}

entrypoint!(process_instruction);

security_txt! {
    name: "ORE",
    project_url: "https://ore.supply",
    contacts: "email:hardhatchad@gmail.com,discord:hardhatchad",
    policy: "https://github.com/regolith-labs/ore/blob/master/SECURITY.md",
    preferred_languages: "en",
    source_code: "https://github.com/regolith-labs/ore"
    // source_revision: default_env!("GITHUB_SHA", ""),
    // source_release: default_env!("GITHUB_REF_NAME", ""),
    // auditors: "None"
}
