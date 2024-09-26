use ore_api::{consts::*, error::OreError, instruction::*, loaders::*, state::Proof};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};
use steel::*;

/// Claim distributes claimable ORE from the treasury to a miner.
pub fn process_claim(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Claim::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer, beneficiary_info, proof_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer.is_signer()?;
    load_token_account(beneficiary_info, None, &MINT_ADDRESS, true)?;
    load_proof(proof_info, signer.key, true)?;
    treasury_info.is_treasury()?;
    load_treasury_tokens(treasury_tokens_info, true)?;
    token_program.has_address(&spl_token::ID)?;

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
