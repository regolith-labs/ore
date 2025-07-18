use ore_api::prelude::*;
use steel::*;

/// Closes a block.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, miner_info, miner_tokens_info, mint_info, opener_info, recipient_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.end_slot + MINING_WINDOW)?; // Window for submitting hashes has closed
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    opener_info.is_writable()?.has_address(&block.opener)?;
    treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info
        .is_writable()?
        .as_associated_token_account(treasury_info.key, mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Load miner rewards.
    if miner_tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            miner_info,
            miner_tokens_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        miner_tokens_info.as_associated_token_account(&miner_info.key, &mint_info.key)?;
    }

    // Payout block reward to winning miner.
    if block.best_hash_miner != Pubkey::default() {
        // Load recipient.
        recipient_info.as_associated_token_account(&block.best_hash_miner, &mint_info.key)?;
        let miner = miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.authority == block.best_hash_miner)?;

        // Update stats.
        miner.total_rewards += block.reward;

        // Transfer reward to miner.
        transfer_signed(
            treasury_info,
            treasury_tokens_info,
            miner_tokens_info,
            token_program,
            block.reward,
            &[TREASURY],
        )?;
    } else {
        // If no one won, burn the block reward.
        burn_signed(
            treasury_tokens_info,
            mint_info,
            treasury_info,
            token_program,
            block.reward,
            &[TREASURY],
        )?;
    }

    // Close block.
    block_info.close(opener_info)?;

    Ok(())
}
