use ore_api::prelude::*;
use steel::*;

/// No-op, use instruction data for logging w/o truncation.
pub fn process_log(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.as_account::<Block>(&ore_api::ID)?;

    // For data integrity, only a block can log messages.

    Ok(())
}
