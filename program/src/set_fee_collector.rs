use ore_api::prelude::*;
use steel::*;

/// Sets the fee collector.
pub fn process_set_fee_collector(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetFeeCollector::try_from_bytes(data)?;
    let new_fee_collector = Pubkey::new_from_array(args.fee_collector);

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
    config.fee_collector = new_fee_collector;

    Ok(())
}
