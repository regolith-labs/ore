use ore_api::prelude::*;
use steel::*;

/// Sets the admin.
pub fn process_migrate_miner(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Set seeker activation flag.
    config.is_seeker_activation_enabled = 0;

    // Set seeker flag.
    miner.is_seeker = 0;
    miner.buffer = [0; 24];

    Ok(())
}
