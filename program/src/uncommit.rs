use ore_api::prelude::*;
use steel::*;

/// Uncommit from a block.
pub fn process_uncommit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Uncommit::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, commitment_info, market_info, miner_info, mint_info, permit_info, recipient_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account::<Block>(&ore_api::ID)?
        .assert(|b| clock.slot < b.start_slot)?;
    commitment_info
        .is_writable()?
        .as_associated_token_account(block_info.key, mint_info.key)?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.id == block.id)?;
    miner_info
        .as_account::<Miner>(&ore_api::ID)?
        .assert(|m| m.authority == *signer_info.key)?;
    mint_info.has_address(&market.base.mint)?.as_mint()?;
    let permit = permit_info
        .as_account_mut::<Permit>(&ore_api::ID)?
        .assert_mut(|p| p.authority == *signer_info.key)?
        .assert_mut(|p| p.block_id == block.id)?;
    recipient_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Normalize amount.
    let amount = permit.amount.min(amount);

    // Transfer hash tokens.
    transfer_signed(
        block_info,
        commitment_info,
        recipient_info,
        token_program,
        amount,
        &[BLOCK, &block.id.to_le_bytes()],
    )?;

    // Update block.
    permit.amount -= amount;

    // Close permit account, if empty.
    if permit.amount == 0 {
        permit_info.close(signer_info)?;
    }

    // Emit event.
    UncommitEvent {
        authority: *signer_info.key,
        block_id: block.id,
        commitment: permit.amount,
        amount,
        ts: clock.unix_timestamp,
    }
    .log_return();

    Ok(())
}
