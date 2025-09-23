use ore_api::prelude::*;
use steel::*;

/// Sets the executor.
pub fn process_set_executor(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = SetExecutor::try_from_bytes(data)?;
    let new_executor = Pubkey::new_from_array(args.executor);

    // Load accounts.
    let [signer_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut_err(
            |m| m.authority == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    system_program.is_program(&system_program::ID)?;

    // Set executor.
    miner.executor = new_executor;

    Ok(())
}
