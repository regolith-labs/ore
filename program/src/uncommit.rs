use ore_api::{prelude::*, sdk::program_log};
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
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot < b.start_slot)?;
    commitment_info
        .is_writable()?
        .has_address(&commitment_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_info.key)?
        .assert(|t| t.owner() == *block_info.key)?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.id == block.id)?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
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
    let amount = permit.commitment.min(amount);

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
    permit.commitment -= amount;
    miner.total_committed -= amount;
    block.total_committed -= amount;

    // Close permit account, if empty.
    if permit.commitment == 0 {
        permit_info.close(signer_info)?;
    }

    // Emit event.
    program_log(
        block.id,
        block_info.clone(),
        &UncommitEvent {
            disc: OreEvent::Uncommit as u64,
            authority: *signer_info.key,
            block_id: block.id,
            block_commitment: block.total_committed,
            permit_commitment: permit.commitment,
            amount,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

    Ok(())
}
