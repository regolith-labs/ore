use entropy_api::state::Var;
use ore_api::prelude::*;
use steel::*;

/// Sets the swap program.
pub fn process_set_var_address(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, board_info, config_info, new_var_address_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    board_info.as_account::<Board>(&ore_api::ID)?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    new_var_address_info
        .as_account::<Var>(&entropy_api::ID)?
        .assert(|v| v.authority == *board_info.key)?
        .assert(|v| v.samples > 0)?;

    // Set var address.
    config.var_address = *new_var_address_info.key;

    Ok(())
}
