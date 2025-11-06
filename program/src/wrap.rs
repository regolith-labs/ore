use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

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
        .assert(|c| c.bury_authority == *signer_info.key)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_sol_info
        .is_writable()?
        .as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    system_program.is_program(&system_program::ID)?;

    // Get amount
    let one_sol = 1_000_000_000;
    let amount = (one_sol * 30).min(treasury.balance);

    // Send SOL to the WSOL account.
    treasury_info.send(amount, treasury_sol_info);

    // Check min balance.
    let min_balance = Rent::get()?.minimum_balance(std::mem::size_of::<Treasury>());
    assert!(
        treasury_info.lamports() >= min_balance,
        "Insufficient SOL balance"
    );

    // Update treasury.
    treasury.balance -= amount;

    Ok(())
}
