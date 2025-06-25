use ore_api::prelude::*;
use steel::*;

/// Sets the admin.
pub fn process_set_admin(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetAdmin::try_from_bytes(data)?;
    let new_admin = Pubkey::new_from_array(args.admin);

    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    system_program.is_program(&system_program::ID)?;

    // Load config account.
    let config = if config_info.data_is_empty() {
        // Assert signer is admin.
        // signer_info.has_address(&ADMIN_ADDRESS)?;

        // Create config account.
        create_program_account::<Config>(
            config_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[CONFIG],
        )?;
        let config = config_info.as_account_mut::<Config>(&ore_api::ID)?;
        config.admin = *signer_info.key;
        config.block_limit = 100;
        config.fee_collector = *signer_info.key;
        config.fee_rate = 0;
        config
    } else {
        config_info
            .as_account_mut::<Config>(&ore_api::ID)?
            .assert_mut(|c| c.admin == *signer_info.key)?
    };

    // Set admin.
    config.admin = new_admin;

    Ok(())
}
