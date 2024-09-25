use ore_api::{consts::*, error::OreError, instruction::Stake};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_pack::Pack,
};
use spl_token::state::Mint;
use steel::*;

/// Upgrade allows a user to migrate a v1 token to a v2 token at a 1:1 exchange rate.
pub fn process_upgrade(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args
    let args = Stake::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer, beneficiary_info, mint_info, mint_v1_info, sender_info, treasury_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_token_account(beneficiary_info, Some(&signer.key), &MINT_ADDRESS, true)?;
    load_mint(mint_info, MINT_ADDRESS, true)?;
    load_mint(mint_v1_info, MINT_V1_ADDRESS, true)?;
    load_token_account(sender_info, Some(signer.key), &MINT_V1_ADDRESS, true)?;
    load_program(token_program, spl_token::id())?;

    // Burn v1 tokens
    solana_program::program::invoke(
        &spl_token::instruction::burn(
            &spl_token::id(),
            sender_info.key,
            mint_v1_info.key,
            signer.key,
            &[signer.key],
            amount,
        )?,
        &[
            token_program.clone(),
            sender_info.clone(),
            mint_v1_info.clone(),
            signer.clone(),
        ],
    )?;

    // Account for decimals change.
    // v1 token has 9 decimals. v2 token has 11.
    let amount_to_mint = amount.saturating_mul(100);

    // Cap at max supply.
    let mint_data = mint_info.data.borrow();
    let mint = Mint::unpack(&mint_data)?;
    if mint.supply.saturating_add(amount_to_mint).gt(&MAX_SUPPLY) {
        return Err(OreError::MaxSupply.into());
    }

    // Mint to the beneficiary account
    drop(mint_data);
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
