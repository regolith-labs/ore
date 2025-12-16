use std::u64;

use ore_api::prelude::*;
use solana_program::log::sol_log;
use spl_token::amount_to_ui_amount;
use steel::*;

/// Amount paid to bots for auto-compounding, in lamports.
const COMPOUND_FEE_PER_TRANSACTION: u64 = 7_000;

/// Compounds yield from the staking contract.
pub fn process_compound_yield(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, mint_info, stake_info, stake_tokens_info, treasury_info, treasury_tokens_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let stake = stake_info
        .as_account_mut::<Stake>(&ore_api::ID)?
        .assert_mut(|s| s.compound_fee_reserve >= COMPOUND_FEE_PER_TRANSACTION)?
        .assert_mut(|s| s.last_claim_at + ONE_DAY < clock.unix_timestamp)?;
    stake_tokens_info
        .is_writable()?
        .as_associated_token_account(stake_info.key, mint_info.key)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    let treasury_tokens = treasury_tokens_info
        .is_writable()?
        .as_associated_token_account(&treasury_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Claim yield from stake account.
    let amount = stake.claim(u64::MAX, &clock, treasury);

    // Deposit into stake account.
    let amount = stake.deposit(amount, &clock, treasury, &treasury_tokens);

    // Transfer ORE from treasury to stake.
    transfer_signed(
        treasury_info,
        treasury_tokens_info,
        stake_tokens_info,
        token_program,
        amount,
        &[TREASURY],
    )?;

    // Deduct compound fee from stake account.
    stake.compound_fee_reserve -= COMPOUND_FEE_PER_TRANSACTION;
    stake_info.send(COMPOUND_FEE_PER_TRANSACTION, &signer_info);

    // Log claim.
    sol_log(
        &format!(
            "Compounding {} ORE",
            amount_to_ui_amount(amount, TOKEN_DECIMALS)
        )
        .as_str(),
    );

    Ok(())
}
