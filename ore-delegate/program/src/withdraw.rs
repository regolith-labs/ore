use ore_delegate_api::prelude::*;
use steel::*;

/// Withdraws hash tokens from the delegate.
pub fn process_withdraw(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Load data.
    let args = Withdraw::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, delegate_info, escrow_info, mint_info, recipient_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let delegate = delegate_info
        .as_account_mut::<Delegate>(&ore_delegate_api::ID)?
        .assert_mut(|d| d.authority == *signer_info.key)?;
    let escrow = escrow_info.as_associated_token_account(delegate_info.key, &mint_info.key)?;
    recipient_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Update delegate.
    delegate.balance -= escrow.amount().min(amount);

    // Transfer tokens.
    transfer_signed(
        signer_info,
        escrow_info,
        recipient_info,
        token_program,
        amount,
        &[
            DELEGATE,
            &delegate.authority.to_bytes(),
            &delegate.block_id.to_le_bytes(),
        ],
    )?;

    // Close accounts if empty.
    if delegate.balance == 0 {
        delegate_info.close(signer_info)?;
        close_token_account_signed(
            escrow_info,
            signer_info,
            delegate_info,
            token_program,
            &[
                DELEGATE,
                &delegate.authority.to_bytes(),
                &delegate.block_id.to_le_bytes(),
            ],
        )?;
    }

    Ok(())
}
