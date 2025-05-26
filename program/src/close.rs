use ore_api::prelude::*;
use steel::*;

/// Close a wager account.
pub fn process_close(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, block_info, wager_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info.as_account::<Block>(&ore_api::ID)?;
    wager_info
        .as_account_mut::<Wager>(&ore_api::ID)?
        .assert_mut(|w| w.authority == *signer_info.key)?
        .assert_mut(|w| w.round < block.current_round)?;
    system_program.is_program(&system_program::ID)?;

    // Close the wager account
    wager_info.close(&signer_info)?;

    Ok(())
}
