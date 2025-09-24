use ore_api::prelude::*;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use steel::*;

const TOKEN_B_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

/// Send SOL from the treasury to the WSOL account.
pub fn process_wrap(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, treasury_info, treasury_sol_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info
        .as_account::<Config>(&ore_api::ID)?
        .assert(|c| c.admin == *signer_info.key)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_sol_info
        .is_writable()?
        .as_associated_token_account(treasury_info.key, &TOKEN_B_MINT)?;
    system_program.is_program(&system_program::ID)?;

    // Send SOL to the WSOL account.
    treasury_info.send(treasury.balance, treasury_sol_info);

    // Update treasury.
    treasury.balance = 0;

    Ok(())
}
