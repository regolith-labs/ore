use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    instruction::StakeArgs, loaders::*, state::Treasury, utils::AccountDeserialize, MINT_ADDRESS,
    MINT_V1_ADDRESS, TREASURY,
};

pub fn process_upgrade<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = StakeArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer_info, beneficiary_info, mint_info, mint_v1_info, sender_info, treasury_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer_info)?;
    load_token_account(
        beneficiary_info,
        Some(&signer_info.key),
        &MINT_ADDRESS,
        true,
    )?;
    load_mint(mint_info, MINT_ADDRESS, true)?;
    load_mint(mint_v1_info, MINT_V1_ADDRESS, true)?;
    load_token_account(sender_info, Some(signer_info.key), &MINT_V1_ADDRESS, true)?;
    load_program(token_program, spl_token::id())?;

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

    // Mint to the beneficiary account
    let treasury_data = treasury_info.data.borrow();
    let treasury = Treasury::try_from_bytes(&treasury_data)?;
    let treasury_bump = treasury.bump as u8;
    drop(treasury_data);
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
        &[&[TREASURY, &[treasury_bump]]],
    )?;

    Ok(())
}
