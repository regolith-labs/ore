use ore_api::prelude::*;
use steel::*;

/// Sets the admin.
pub fn process_migrate_staker(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, mint_info, stake_info, stake_tokens_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info.as_account::<Config>(&ore_api::ID)?.assert_err(
        |c| c.admin == *signer_info.key,
        OreError::NotAuthorized.into(),
    )?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let stake = stake_info.as_account_mut::<Stake>(&ore_api::ID)?;
    stake_tokens_info.is_empty()?.is_writable()?;
    treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info.as_associated_token_account(&treasury_info.key, &MINT_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Create stake tokens account.
    if stake_tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            stake_info,
            stake_tokens_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    }
    let stake_tokens =
        stake_tokens_info.as_associated_token_account(stake_info.key, mint_info.key)?;

    // Move tokens from treasury to stake tokens.
    if stake_tokens.amount() == 0 {
        transfer_signed(
            treasury_info,
            treasury_tokens_info,
            stake_tokens_info,
            token_program,
            stake.balance,
            &[TREASURY],
        )?;
    }

    // Safety check.
    let stake_tokens =
        stake_tokens_info.as_associated_token_account(stake_info.key, mint_info.key)?;
    assert_eq!(stake_tokens.amount(), stake.balance);

    Ok(())
}
