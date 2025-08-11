use ore_api::prelude::*;
use steel::*;

/// Sets the sniper fee duration.
pub fn process_set_sniper_fee_duration(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetSniperFeeDuration::try_from_bytes(data)?;
    let new_sniper_fee_duration = u64::from_le_bytes(args.sniper_fee_duration);

    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut(|c| c.admin == *signer_info.key)?;
    system_program.is_program(&system_program::ID)?;

    // Set fee rate.
    config.sniper_fee_duration = new_sniper_fee_duration;

    Ok(())
}
