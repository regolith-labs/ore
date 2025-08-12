use ore_api::prelude::*;
use steel::*;

/// Sets the block duration.
pub fn process_set_block_duration(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetBlockDuration::try_from_bytes(data)?;
    let new_block_duration = u64::from_le_bytes(args.block_duration);

    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut(|c| c.admin == *signer_info.key)?;
    system_program.is_program(&system_program::ID)?;

    // Set fee collector.
    config.block_duration = new_block_duration;

    Ok(())
}
