use ore_api::prelude::*;
use steel::*;

/// Deposits stake.
pub fn process_deposit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Deposit::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, miner_info, sender_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
    sender_info.as_associated_token_account(&signer_info.key, &MINT_ADDRESS)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info.as_associated_token_account(&treasury_info.key, &MINT_ADDRESS)?;
    token_program.is_program(&spl_token::ID)?;

    // Update account state.
    miner.stake += amount;
    treasury.total_stake += amount;

    // Execute transfer.
    transfer(
        signer_info,
        sender_info,
        treasury_tokens_info,
        token_program,
        amount,
    )?;

    Ok(())
}
