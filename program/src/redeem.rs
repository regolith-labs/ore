use ore_api::prelude::*;
use steel::*;

/// Redeem ORE for SOL backing.
pub fn process_redeem(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Redeem::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, mint_info, sender_info, treasury_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let mint = mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let sender = sender_info.as_associated_token_account(&signer_info.key, &mint_info.key)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Normalize amount.
    let amount = amount.min(sender.amount());

    // Load redemption amount.
    let redemption_amount = treasury.balance * amount / mint.supply();

    // Burn ORE.
    burn(sender_info, mint_info, signer_info, token_program, amount)?;

    // Transfer SOL to recipient.
    assert!(
        treasury.balance >= redemption_amount,
        "Redemption too large"
    );
    treasury_info.send(redemption_amount, signer_info);
    treasury.balance -= redemption_amount;

    Ok(())
}
