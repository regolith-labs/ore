use ore_api::{prelude::*, sdk::program_log};
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
    mint_ore_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    recipient_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_ore_info.key)?;
    let stake = stake_info
        .as_account_mut::<Stake>(&ore_api::ID)?
        .assert_mut(|p| p.authority == *signer_info.key)?;
    block_info.has_seeds(&[BLOCK, &stake.block_id.to_le_bytes()], &ore_api::ID)?;
    collateral_info
        .is_writable()?
        .has_address(&collateral_pda(stake.block_id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_ore_info.key)?
        .assert(|t| t.owner() == *block_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Check timestamp. Collateral can only be withdrawn after the block has started mining
    let start_slot = stake.block_id * 1500;
    if clock.slot < start_slot {
        return Err(ProgramError::InvalidArgument);
    }

    // Update stake state.
    stake.collateral -= amount;

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
    if stake.collateral == 0 {
        stake_info.close(signer_info)?;
    }

    // Emit event.
    program_log(
        stake.block_id,
        block_info.clone(),
        &WithdrawEvent {
            disc: OreEvent::Withdraw as u64,
            authority: *signer_info.key,
            block_id: stake.block_id,
            amount,
            collateral: stake.collateral,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

    Ok(())
}
