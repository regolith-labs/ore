use ore_api::prelude::*;
use steel::*;

/// Sets the fee rate.
pub fn process_set_fee_rate(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetFeeRate::try_from_bytes(data)?;
    let new_fee_rate = u64::from_le_bytes(args.fee_rate);

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
    config.fee_rate = new_fee_rate;

    Ok(())
}
