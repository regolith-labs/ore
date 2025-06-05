use ore_api::prelude::*;
use steel::*;

/// Buy hashpower.
pub fn process_buy(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, market_info, miner_info, recipient_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot + 1500)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // TODO Buy hash tokens

    Ok(())
}
