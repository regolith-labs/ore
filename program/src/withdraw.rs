use ore_api::prelude::*;
use solana_program::log::sol_log;
use spl_token::amount_to_ui_amount;
use steel::*;

use crate::AUTHORIZED_ACCOUNTS;

/// Withdraws ORE from the staking contract.
pub fn process_withdraw(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Withdraw::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, mint_info, recipient_info, stake_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    recipient_info
        .is_writable()?
        .as_associated_token_account(&signer_info.key, &mint_info.key)?;
    let stake = stake_info
        .as_account_mut::<Stake>(&ore_api::ID)?
        .assert_mut(|s| s.authority == *signer_info.key)?
        .assert_mut(|s| clock.unix_timestamp > s.last_deposit_at + ONE_HOUR)?; // Must wait one hour since last deposit
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info
        .is_writable()?
        .as_associated_token_account(&treasury_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Check whitelist
    if !AUTHORIZED_ACCOUNTS.contains(&signer_info.key) {
        return Err(trace("Not authorized", OreError::NotAuthorized.into()));
    }

    // Open recipient token account.
    if recipient_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            signer_info,
            recipient_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    }

    // Deposit into stake account.
    let amount = stake.withdraw(amount, &clock, treasury);

    // Transfer ORE to recipient.
    transfer_signed(
        treasury_info,
        treasury_tokens_info,
        recipient_info,
        token_program,
        amount,
        &[TREASURY],
    )?;

    // Log withdraw.
    sol_log(
        &format!(
            "Withdrawing {} ORE",
            amount_to_ui_amount(amount, TOKEN_DECIMALS)
        )
        .as_str(),
    );

    Ok(())
}
