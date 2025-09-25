use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Sets the admin.
pub fn process_migrate_squares(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, squares_info, treasury_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let squares_old = squares_info.as_account_mut::<SquareOLD>(&ore_api::ID)?;
    let treasury_old = treasury_info.as_account_mut::<TreasuryOLD>(&ore_api::ID)?;
    config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Record old values.
    let count = squares_old.count;
    let miners = squares_old.miners;

    // Record old values.
    let balance = treasury_old.balance;

    // Realloc squares
    let new_size = 8 + std::mem::size_of::<Square>();
    let old_size = 8 + std::mem::size_of::<SquareOLD>();
    let new_rent = Rent::get()?.minimum_balance(new_size);
    let old_rent = Rent::get()?.minimum_balance(old_size);
    let additional_rent = new_rent - old_rent;
    squares_info.realloc(new_size, false)?;
    squares_info.collect(additional_rent, &signer_info)?;

    // Realloc treasury
    let new_size = 8 + std::mem::size_of::<Treasury>();
    let old_size = 8 + std::mem::size_of::<TreasuryOLD>();
    let new_rent = Rent::get()?.minimum_balance(new_size);
    let old_rent = Rent::get()?.minimum_balance(old_size);
    let additional_rent = new_rent - old_rent;
    treasury_info.realloc(new_size, false)?;
    treasury_info.collect(additional_rent, &signer_info)?;

    // Update squares.
    let squares = squares_info.as_account_mut::<Square>(&ore_api::ID)?;
    squares.count = count;
    squares.miners = miners;
    squares.deployed = [[0; 16]; 25];

    // Update treasury.
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury.balance = balance;
    treasury.motherlode = 0;

    Ok(())
}
