use ore_api::prelude::*;
use steel::*;

/// Initializes the program.
pub fn process_initialize_squares(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let (required_accounts, square_accounts) = accounts.split_at(2);
    let [signer_info, system_program] = required_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;

    // Create config account.
    for i in 0..25 {
        let square_info = &square_accounts[i];
        if square_info.data_is_empty() {
            create_program_account::<Square>(
                square_info,
                system_program,
                signer_info,
                &ore_api::ID,
                &[SQUARE, &(i as u64).to_le_bytes()],
            )?;
            let square = square_info.as_account_mut::<Square>(&ore_api::ID)?;
            square.id = i as u64;
            square.round_id = 0;
            square.miners = [Pubkey::default(); 16];
        } else {
            square_info
                .as_account_mut::<Square>(&ore_api::ID)?
                .assert_mut(|s| s.id == i as u64)?;
        }
    }

    Ok(())
}
