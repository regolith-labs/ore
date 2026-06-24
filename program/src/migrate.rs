use ore_api::prelude::*;
use solana_program::{pubkey, rent::Rent};
use steel::*;

const ADMIN_ADDRESS: Pubkey = pubkey!("EqbHxJd7UJDjDnZtbbgjo4egidbNgvSjttn1NHmF4aos");

/// Migrates the config account.
pub fn process_migrate(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, automation_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.has_address(&ADMIN_ADDRESS)?.is_signer()?;
    // let automation = automation_info.as_account_mut::<AutomationV1>(&ore_api::ID)?;
    // system_program.is_program(&system_program::ID)?;

    // Record old data.
    // let amount = automation.amount;
    // let authority = automation.authority;
    // let balance = automation.balance;
    // let executor = automation.executor;
    // let fee = automation.fee;
    // let strategy = automation.strategy;
    // let mask = automation.mask;
    // let reload = automation.reload;

    // Migrate the config account.
    // let old_size = AutomationV1::SIZE;
    // let new_size = AutomationV4::SIZE;
    // let old_rent = Rent::get()?.minimum_balance(old_size);
    // let new_rent = Rent::get()?.minimum_balance(new_size);
    // let lamports = new_rent - old_rent;

    // Transfer the required rent to the account.
    // automation_info.realloc(new_size, true)?;
    // automation_info.collect(lamports, signer_info)?;

    // Update account state.
    // let automation = automation_info.as_account_mut::<AutomationV4>(&ore_api::ID)?;
    // automation.amount = amount;
    // automation.authority = authority;
    // automation.balance = balance;
    // automation.executor = executor;
    // automation.fee = fee;
    // automation.strategy = strategy;
    // automation.mask = mask;
    // automation.reload = reload;
    // automation.total_sol_spent = 0;
    // automation.total_ore_earned = 0;
    // automation.conditions = AutomationConditions::default();

    Ok(())
}
