use ore_api::prelude::*;
use steel::*;

/// Close a commit account.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, block_info, commit_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info.as_account::<Block>(&ore_api::ID)?;
    commit_info
        .as_account_mut::<Commit>(&ore_api::ID)?
        .assert_mut(|c| c.authority == *signer_info.key)?
        .assert_mut(|c| c.round < block.current_round)?;
    system_program.is_program(&system_program::ID)?;

    // Close the commit account
    commit_info.close(&signer_info)?;

    Ok(())
}
