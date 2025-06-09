use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Withdraws collateral.
pub fn process_withdraw(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Withdraw::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, collateral_info, mint_ore_info, recipient_info, stake_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    collateral_info
        .is_writable()?
        .as_associated_token_account(block_info.key, mint_ore_info.key)?;
    mint_ore_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    recipient_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_ore_info.key)?;
    let stake = stake_info
        .as_account_mut::<Stake>(&ore_api::ID)?
        .assert_mut(|p| p.authority == *signer_info.key)?;
    block_info.has_seeds(&[BLOCK, &stake.block_id.to_le_bytes()], &ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Check timestamp. Collateral can only be withdrawn after the block has started mining
    let start_slot = stake.block_id * 1500;
    if clock.slot < start_slot {
        return Err(ProgramError::InvalidArgument);
    }

    // Update stake state.
    stake.capacity -= amount;

    // Transfer collateral.
    transfer_signed(
        block_info,
        collateral_info,
        recipient_info,
        token_program,
        amount,
        &[BLOCK, &stake.block_id.to_le_bytes()],
    )?;

    // Close stake account, if empty.
    if stake.capacity == 0 {
        stake_info.close(signer_info)?;
    }

    Ok(())
}
