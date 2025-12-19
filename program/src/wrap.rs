use ore_api::prelude::*;
use solana_program::{native_token::LAMPORTS_PER_SOL, rent::Rent};
use steel::*;

/// Send SOL from the treasury to the WSOL account.
pub fn process_wrap(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Wrap::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, _config_info, treasury_info, treasury_sol_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&BURY_AUTHORITY)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_sol_info
        .is_writable()?
        .as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    system_program.is_program(&system_program::ID)?;

    // Get amount
    let amount = (LAMPORTS_PER_SOL * 100).min(treasury.balance).min(amount);

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
