use ore_api::prelude::*;
use steel::*;

/// Commit to a block.
pub fn process_commit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Commit::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, commitment_info, market_info, miner_info, mint_info, permit_info, sender_info, system_program, token_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account::<Block>(&ore_api::ID)?
        .assert(|b| clock.slot < b.start_slot)?;
    commitment_info.as_associated_token_account(block_info.key, mint_info.key)?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.id == block.id)?;
    miner_info
        .as_account::<Miner>(&ore_api::ID)?
        .assert(|m| m.authority == *signer_info.key)?;
    mint_info.has_address(&market.base.mint)?.as_mint()?;
    let sender = sender_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Normalize amount.
    let amount = sender.amount().min(amount);

    // Load permit account.
    let permit = if permit_info.data_is_empty() {
        create_program_account::<Permit>(
            permit_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[PERMIT, &signer_info.key.to_bytes(), &block.id.to_le_bytes()],
        )?;
        let permit = permit_info.as_account_mut::<Permit>(&ore_api::ID)?;
        permit.amount = 0;
        permit.authority = *signer_info.key;
        permit.block_id = block.id;
        permit
    } else {
        permit_info
            .as_account_mut::<Permit>(&ore_api::ID)?
            .assert_mut(|p| p.authority == *signer_info.key)?
            .assert_mut(|p| p.block_id == block.id)?
    };

    // Transfer hash tokens.
    transfer(
        signer_info,
        sender_info,
        commitment_info,
        token_program,
        amount,
    )?;

    // Update block.
    permit.amount += amount;

    Ok(())
}
