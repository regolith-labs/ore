use ore_api::prelude::*;
use steel::*;

/// Sets whether admin supplied RNG is enabled.
pub fn process_set_is_new_rng_enabled(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetIsNewRngEnabled::try_from_bytes(data)?;
    let is_new_rng_enabled = args.is_new_rng_enabled;

    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Set fee collector.
    config.is_new_rng_enabled = is_new_rng_enabled as u64;

    Ok(())
}
