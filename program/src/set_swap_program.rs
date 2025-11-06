use ore_api::prelude::*;
use steel::*;

/// Sets the swap program.
pub fn process_set_swap_program(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, new_swap_program_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    new_swap_program_info.is_executable()?;

    // Set swap program.
    config.swap_program = *new_swap_program_info.key;

    Ok(())
}
