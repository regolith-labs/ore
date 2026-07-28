use ore_api::prelude::*;
use solana_program::pubkey;
use steel::*;

const ADMIN: Pubkey = pubkey!("EqbHxJd7UJDjDnZtbbgjo4egidbNgvSjttn1NHmF4aos");

/// Sets the admin.
pub fn process_migrate(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, automation_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN)?;
    let automation = automation_info.as_account_mut::<Automation>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Set admin.
    automation.conditions = AutomationConditions::default();

    Ok(())
}
