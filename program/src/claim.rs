use coal_api::{consts::*, error::OreError, instruction::ClaimArgs, loaders::*, state::{Proof, ProofV2}};
use coal_utils::{Discriminator, spl::transfer_signed};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::utils::AccountDeserialize;

/// Claim distributes claimable ORE from the treasury to a miner.
pub fn process_claim<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let proof_info = &accounts[2];

    if proof_info.data.borrow()[0].eq(&(Proof::discriminator() as u8)) {
        return process_claim_coal(accounts, data)
    }

    if proof_info.data.borrow()[0].eq(&(ProofV2::discriminator() as u8)) {
        return process_claim_wood(accounts, data)
    }

    return Err(solana_program::program_error::ProgramError::InvalidAccountData);
}

fn process_claim_coal(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = ClaimArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer, beneficiary_info, proof_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_token_account(beneficiary_info, None, &COAL_MINT_ADDRESS, true)?;
    load_coal_proof(proof_info, signer.key, true)?;
    load_treasury(treasury_info, false)?;
    load_coal_treasury_tokens(treasury_tokens_info, true)?;
    load_program(token_program, spl_token::id())?;

    // Update miner balance.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.balance = proof
        .balance
        .checked_sub(amount)
        .ok_or(OreError::ClaimTooLarge)?;

    // Transfer tokens from treasury to beneficiary.
    transfer_signed(
        treasury_info,
        treasury_tokens_info,
        beneficiary_info,
        token_program,
        amount,
        &[&[TREASURY, &[TREASURY_BUMP]]],
    )?;

    Ok(())
}

fn process_claim_wood(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = ClaimArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer, beneficiary_info, proof_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_token_account(beneficiary_info, None, &WOOD_MINT_ADDRESS, true)?;
    load_proof_v2(proof_info, signer.key, &WOOD_MINT_ADDRESS, true)?;
    load_treasury(treasury_info, false)?;
    load_wood_treasury_tokens(treasury_tokens_info, true)?;
    load_program(token_program, spl_token::id())?;

    // Update miner balance.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = ProofV2::try_from_bytes_mut(&mut proof_data)?;
    proof.balance = proof
        .balance
        .checked_sub(amount)
        .ok_or(OreError::ClaimTooLarge)?;

    // Transfer tokens from treasury to beneficiary.
    transfer_signed(
        treasury_info,
        treasury_tokens_info,
        beneficiary_info,
        token_program,
        amount,
        &[&[TREASURY, &[TREASURY_BUMP]]],
    )?;

    Ok(())
}