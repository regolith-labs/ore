use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    error::OreError,
    instruction::ClaimArgs,
    loaders::*,
    state::{Proof, Treasury},
    TREASURY,
};

pub fn process_claim<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = bytemuck::try_from_bytes::<ClaimArgs>(data)
        .or(Err(ProgramError::InvalidInstructionData))?;

    // Validate accounts
    let [signer, beneficiary_info, mint_info, proof_info, treasury_info, treasury_tokens_info, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_token_account(beneficiary_info, None, mint_info.key, true)?;
    load_mint(mint_info, true)?;
    load_treasury(treasury_info, true)?;
    load_token_account(
        treasury_tokens_info,
        Some(treasury_info.key),
        mint_info.key,
        true,
    )?;
    load_program(token_program, spl_token::id())?;

    // Validate claim amout
    let mut proof_data = proof_info.data.borrow_mut();
    let mut proof = bytemuck::try_from_bytes_mut::<Proof>(&mut proof_data).unwrap();
    if proof.claimable_rewards.lt(&args.amount) {
        return Err(OreError::InvalidClaimAmount.into());
    }

    // Update claimable amount
    proof.claimable_rewards = proof.claimable_rewards.saturating_sub(args.amount);

    // Update lifetime status
    let mut treasury_data = treasury_info.data.borrow_mut();
    let mut treasury = bytemuck::try_from_bytes_mut::<Treasury>(&mut treasury_data).unwrap();
    treasury.total_claimed_rewards = treasury.total_claimed_rewards.saturating_add(args.amount);

    // Distribute tokens from treasury to beneficiary
    solana_program::program::invoke_signed(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            treasury_tokens_info.key,
            beneficiary_info.key,
            treasury_info.key,
            &[treasury_info.key],
            args.amount,
        )?,
        &[
            token_program.clone(),
            treasury_tokens_info.clone(),
            beneficiary_info.clone(),
            treasury_info.clone(),
        ],
        &[&[TREASURY, &[treasury.bump as u8]]],
    )?;

    Ok(())
}
