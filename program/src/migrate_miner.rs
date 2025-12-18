use ore_api::prelude::*;
use steel::*;

/// Migrates miner lifetime deployed amount.
pub fn process_migrate_miner(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = MigrateMiner::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, config_info, miner_info, migration_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
    migration_info
        .is_writable()?
        .is_empty()?
        .has_seeds(&[MIGRATION, &miner.authority.as_ref()], &ore_api::ID)?;

    // Update lifetime deployed amount.
    miner.lifetime_deployed += amount;

    create_program_account::<Migration>(
        migration_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[MIGRATION, &miner.authority.to_bytes()],
    )?;

    Ok(())
}
