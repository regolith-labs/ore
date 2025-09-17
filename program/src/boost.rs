use ore_api::prelude::*;
use steel::*;

/// Sets the admin.
pub fn process_boost(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, config_info, mint_info, reserve_tokens_info, treasury_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut(|c| clock.unix_timestamp >= c.last_boost + (10 * ONE_MINUTE))?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    reserve_tokens_info
        .has_address(&BOOST_RESERVE_TOKEN)?
        .as_token_account()?
        .assert(|t| t.mint() == MINT_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Update config.
    config.last_boost = clock.unix_timestamp;

    // Mint tokens to the boost program.
    mint_to_signed(
        mint_info,
        reserve_tokens_info,
        treasury_info,
        token_program,
        ONE_ORE * 3,
        &[TREASURY],
    )?;

    Ok(())
}
