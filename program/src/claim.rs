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
    let [signer_info, beneficiary_info, proof_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    beneficiary_info
        .is_writable()?
        .to_token_account()?
        .check(|t| t.mint == MINT_ADDRESS)?;
    let proof = proof_info
        .to_account_mut::<Proof>(&ore_api::ID)?
        .check_mut(|p| p.authority == *signer_info.key)?;
    treasury_info.is_treasury()?;
    treasury_tokens_info.is_writable()?.is_treasury_tokens()?;
    token_program.is_program(&spl_token::ID)?;

    // Update miner balance.
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
