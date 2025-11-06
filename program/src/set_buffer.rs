use ore_api::prelude::*;
use steel::*;

/// Sets the buffer.
pub fn process_set_buffer(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetBuffer::try_from_bytes(data)?;
    let new_buffer = u64::from_le_bytes(args.buffer);

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

    // Set buffer.
    config.buffer = new_buffer;

    Ok(())
}
