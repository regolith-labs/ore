use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Sets the admin.
pub fn process_migrate_automation(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, authority_info, config_info, automation_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    if automation_info.data_is_empty() {
        return Ok(());
    }
    automation_info
        .as_account_mut::<Automation>(&ore_api::ID)?
        .assert_mut(|a| a.authority == *authority_info.key)?;
    system_program.is_program(&system_program::ID)?;

    // Close automation if its out of funds.
    let new_size = 8 + std::mem::size_of::<Automation>();
    let new_rent = Rent::get()?.minimum_balance(new_size);
    if automation_info.lamports() <= new_rent {
        automation_info.close(authority_info)?;
        return Ok(());
    }

    Ok(())
}
