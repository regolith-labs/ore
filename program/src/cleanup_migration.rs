use ore_api::prelude::*;
use steel::*;

/// Cleans up the migration account.
pub fn process_cleanup_migration(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, migration_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    migration_info.as_account_mut::<Migration>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Close migration account.
    migration_info.close(signer_info)?;

    Ok(())
}
