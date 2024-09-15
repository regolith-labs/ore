use coal_api::{consts::*, error::CoalError, instruction::ClaimArgs, loaders::*, state::Proof};
use coal_utils::spl::transfer_signed;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::utils::AccountDeserialize;

pub fn process_claim_coal(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
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
        .ok_or(CoalError::ClaimTooLarge)?;

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