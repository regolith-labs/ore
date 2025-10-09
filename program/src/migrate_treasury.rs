use ore_api::prelude::*;
use solana_program::{pubkey, rent::Rent};
use steel::*;

/// Sets the admin.
pub fn process_migrate_treasury(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<ConfigOLD>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Record old values.
    let admin = config.admin;
    let fee_collector = config.fee_collector;
    let last_boost = config.last_boost;
    let is_seeker_activation_enabled = config.is_seeker_activation_enabled;

    // Realloc treasury.
    let new_size = 8 + std::mem::size_of::<Config>();
    let old_size = 8 + std::mem::size_of::<ConfigOLD>();
    let new_rent = Rent::get()?.minimum_balance(new_size);
    let old_rent = Rent::get()?.minimum_balance(old_size);
    config_info.realloc(new_size, false)?;
    if new_size > old_size {
        let additional_rent = new_rent.saturating_sub(old_rent);
        config_info.collect(additional_rent, &signer_info)?;
    }

    // Update config.
    let config = config_info.as_account_mut::<Config>(&ore_api::ID)?;
    config.admin = admin;
    config.fee_collector = fee_collector;
    config.bury_authority = pubkey!("HNWhK5f8RMWBqcA7mXJPaxdTPGrha3rrqUrri7HSKb3T");
    config.last_boost = last_boost;
    config.is_seeker_activation_enabled = is_seeker_activation_enabled;

    Ok(())
}
