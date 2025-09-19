use ore_api::prelude::*;
use steel::*;

/// No-op, use instruction data for logging w/o truncation.
pub fn process_log(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.as_account::<Board>(&ore_api::ID)?;

    // For data integrity, only the board can log messages.

    Ok(())
}
