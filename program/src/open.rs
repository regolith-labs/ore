use ore_api::prelude::*;
use steel::*;

/// Opens a new block.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Open::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);

    // Load accounts.
    let [signer_info, block_info, market_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.block_id < id)?; // Only allow opening blocks in forward bias
    system_program.is_program(&system_program::ID)?;

    // Create block, if it doesn't exist.
    if block_info.data_is_empty() {
        block_info
            .is_empty()? // Account has not been initialized
            .is_writable()? // Account is writable
            .has_seeds(&[BLOCK, &id.to_le_bytes()], &ore_api::ID)?; // Account has correct seeds

        // Create block account.
        create_program_account::<Block>(
            block_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[BLOCK, &id.to_le_bytes()],
        )?;
        let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
        block.id = id;
        block.opener = *signer_info.key;
        block.best_hash = [0; 32];
        block.best_hash_miner = Pubkey::default();
        block.reward = 0; // Set by reset instruction
        block.start_slot = u64::MAX; // Set by reset instruction
        block.end_slot = u64::MAX; // Set by reset instruction
        block.slot_hash = [0; 32]; // Set by reset instruction
        block.total_hashpower = 0;
    } else {
        block_info
            .as_account_mut::<Block>(&ore_api::ID)?
            .assert_mut(|b| b.id == id)?;
    }

    Ok(())
}
