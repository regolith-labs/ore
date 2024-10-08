use ore_api::prelude::*;
use steel::*;

/// Upgrade allows a user to migrate a v1 token to a v2 token at a 1:1 exchange rate.
pub fn process_upgrade(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args
    let args = Stake::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer_info, beneficiary_info, mint_info, mint_v1_info, sender_info, treasury_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    beneficiary_info
        .is_writable()?
        .to_token_account()?
        .check(|t| t.owner == *signer_info.key)?
        .check(|t| t.mint == MINT_ADDRESS)?;
    let mint = mint_info
        .is_writable()?
        .has_address(&MINT_ADDRESS)?
        .to_mint()?;
    mint_v1_info
        .is_writable()?
        .has_address(&MINT_V1_ADDRESS)?
        .to_mint()?;
    sender_info
        .is_writable()?
        .to_token_account()?
        .check(|t| t.owner == *signer_info.key)?
        .check(|t| t.mint == MINT_V1_ADDRESS)?;
    treasury_info.is_treasury()?;
    token_program.is_program(&spl_token::ID)?;

    // Burn v1 tokens
    solana_program::program::invoke(
        &spl_token::instruction::burn(
            &spl_token::id(),
            sender_info.key,
            mint_v1_info.key,
            signer_info.key,
            &[signer_info.key],
            amount,
        )?,
        &[
            token_program.clone(),
            sender_info.clone(),
            mint_v1_info.clone(),
            signer_info.clone(),
        ],
    )?;

    // Account for decimals change.
    // v1 token has 9 decimals. v2 token has 11.
    let amount_to_mint = amount.saturating_mul(100);

    // Cap at max supply.
    if mint.supply.saturating_add(amount_to_mint).gt(&MAX_SUPPLY) {
        return Err(OreError::MaxSupply.into());
    }

    // Mint to the beneficiary account
    mint_to_signed(
        mint_info,
        beneficiary_info,
        treasury_info,
        token_program,
        amount_to_mint,
        &[&[TREASURY, &[TREASURY_BUMP]]],
    )?;

    Ok(())
}
