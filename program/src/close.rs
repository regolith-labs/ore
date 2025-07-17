use ore_api::prelude::*;
use steel::*;

/// Closes a block.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, miner_info, mint_info, opener_info, recipient_info, treasury_info, system_program, token_program, ore_program] =
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
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Payout block reward.
    if block.best_hash_miner != Pubkey::default() {
        // Load recipient.
        recipient_info.as_associated_token_account(&block.best_hash_miner, &mint_info.key)?;
        let miner = miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.authority == block.best_hash_miner)?;

        // Limit payout to supply cap.
        let ore_mint = mint_info.as_mint()?;
        let max_reward = MAX_SUPPLY.saturating_sub(ore_mint.supply());
        let reward_amount = block.reward.min(max_reward);

        // Update stats.
        miner.total_rewards += reward_amount;

        // Mint reward to recipient.
        mint_to_signed(
            mint_info,
            recipient_info,
            treasury_info,
            token_program,
            reward_amount,
            &[TREASURY],
        )?;
    }

    // Close block.
    block_info.close(opener_info)?;

    // Emit event.
    // program_log(
    //     block.id,
    //     &[block_info.clone(), ore_program.clone()],
    //     &CloseEvent {
    //         authority: *signer_info.key,
    //         id: block.id,
    //         burned_base: base_burned,
    //         burned_quote: quote_burned,
    //         ts: clock.unix_timestamp,
    //     }
    //     .to_bytes(),
    // )?;

    Ok(())
}
