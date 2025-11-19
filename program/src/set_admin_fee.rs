use ore_api::prelude::*;
use steel::*;

/// Sets the admin fee.
pub fn process_set_admin_fee(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetAdminFee::try_from_bytes(data)?;
    let new_admin_fee = u64::from_le_bytes(args.admin_fee);

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

    // Cap admin fee at 1%.
    let new_admin_fee = new_admin_fee.min(100);

    // Set admin fee.
    config.admin_fee = new_admin_fee;

    Ok(())
}
