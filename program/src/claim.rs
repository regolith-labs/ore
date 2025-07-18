use ore_api::prelude::*;
use steel::*;

/// Claims a block reward.
pub fn process_claim(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Claim::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, miner_info, miner_tokens_info, recipient_info, mint_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account::<Miner>(&ore_api::ID)?
        .assert(|m| m.authority == *signer_info.key)?;
    let miner_tokens =
        miner_tokens_info.as_associated_token_account(&miner_info.key, &mint_info.key)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Load recipient.
    if recipient_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            signer_info,
            recipient_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        recipient_info.as_associated_token_account(signer_info.key, mint_info.key)?;
    }

    // Load amount.
    let amount = miner_tokens.amount().min(amount);

    // Transfer reward to recipient.
    transfer_signed(
        miner_info,
        miner_tokens_info,
        recipient_info,
        token_program,
        amount,
        &[MINER, miner.authority.as_ref()],
    )?;

    Ok(())
}
