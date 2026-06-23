use ore_api::prelude::*;
use steel::*;

/// Sets the admin.
pub fn process_set_admin(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Parse data.
    // let args = SetAdmin::try_from_bytes(data)?;
    // let new_admin = Pubkey::new_from_array(args.admin);

    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .has_seeds(&[CONFIG], &ore_api::ID)?
        .as_account_mut::<ConfigV4>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin.authority == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Set admin.
    // config.admin.authority = new_admin;

    Ok(())
}
