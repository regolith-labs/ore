use ore_api::{consts::*, error::OreError, instruction::StakeArgs, loaders::*};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_pack::Pack, pubkey::Pubkey,
};
use spl_token::state::Mint;

/// Upgrade allows a user to migrate a v1 token to a v2 token one-for-one. Its responsibilies include:
/// 1. Burns the v1 tokens.
/// 2. Mints an equivalent number of v2 tokens to the user.
///
/// Safety requirements:
/// - Upgrade is a permissionless instruction and can be called by any user.
/// - The provided beneficiary, mint, mint v1, sender, and token program must be valid.
pub fn process_upgrade<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = StakeArgs::try_from_bytes(data)?;
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
    drop(mint_data);

    // Mint to the beneficiary account
    solana_program::program::invoke_signed(
        &spl_token::instruction::mint_to(
            &spl_token::id(),
            mint_info.key,
            beneficiary_info.key,
            treasury_info.key,
            &[treasury_info.key],
            amount_to_mint,
        )?,
        &[
            token_program.clone(),
            mint_info.clone(),
            beneficiary_info.clone(),
            treasury_info.clone(),
        ],
        &[&[TREASURY, &[TREASURY_BUMP]]],
    )?;

    Ok(())
}
