use ore_api::prelude::*;
use ore_delegate_api::prelude::*;
use steel::*;

/// Deposits hash tokens for cranking.
pub fn process_deposit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Load data.
    let args = Deposit::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, block_info, delegate_info, escrow_info, market_info, mint_info, sender_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    let market = market_info
        .as_account_mut::<Market>(&ore_api::ID)?
        .assert_mut(|m| m.id == block.id)?;
    mint_info.has_address(&market.base.mint)?.as_mint()?;
    sender_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &market.base.mint)?
        .assert_mut(|t| t.amount() >= amount)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Initialize delegate.
    let delegate = if delegate_info.data_is_empty() {
        create_program_account::<Delegate>(
            delegate_info,
            system_program,
            signer_info,
            &ore_delegate_api::ID,
            &[
                DELEGATE,
                &signer_info.key.to_bytes(),
                &block.id.to_le_bytes(),
            ],
        )?;
        let delegate = delegate_info.as_account_mut::<Delegate>(&ore_delegate_api::ID)?;
        delegate.authority = *signer_info.key;
        delegate.block_id = block.id;
        delegate.fee = 0; // TODO: Set fee.
        delegate
    } else {
        delegate_info.as_account_mut::<Delegate>(&ore_delegate_api::ID)?
    };

    // Initialize escrow.
    if escrow_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            delegate_info,
            escrow_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        escrow_info
            .is_writable()?
            .as_associated_token_account(delegate_info.key, &mint_info.key)?;
    }

    // Update delegate.
    delegate.balance += amount;

    // Transfer tokens.
    transfer(signer_info, sender_info, escrow_info, token_program, amount)?;

    Ok(())
}
