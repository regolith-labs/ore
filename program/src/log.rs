use ore_api::prelude::*;
use solana_program::log::sol_log;
use steel::*;

/// No-op, use instruction data for logging w/o truncation.
pub fn process_log(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Load data
    let block_id_bytes = data[..8].try_into().unwrap();
    let block_id = u64::from_le_bytes(block_id_bytes);
    sol_log(format!("Block ID: {}", block_id).as_str());

    // Load accounts.
    let [signer_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info
        .is_signer()?
        .has_seeds(&[BLOCK, &block_id.to_le_bytes()], &ore_api::ID)?;

    // For data integrity, only a block can log messages.

    Ok(())
}
