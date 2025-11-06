use ore_api::prelude::*;
use steel::*;

/// Creates a new var account.
pub fn process_new_var(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = NewVar::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);
    let commit = args.commit;
    let samples = u64::from_le_bytes(args.samples);

    // Load accounts.
    let [signer_info, board_info, config_info, provider_info, var_info, system_program, entropy_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info.as_account_mut::<Board>(&ore_api::ID)?;
    config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    entropy_program.is_program(&entropy_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    invoke_signed(
        &entropy_api::sdk::open(
            *board_info.key,
            *signer_info.key,
            id,
            *provider_info.key,
            commit,
            false,
            samples,
            board.end_slot,
        ),
        &[
            board_info.clone(),
            signer_info.clone(),
            provider_info.clone(),
            var_info.clone(),
            system_program.clone(),
        ],
        &entropy_api::ID,
        &[BOARD],
    )?;

    Ok(())
}
