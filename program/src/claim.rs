use ore_api::{consts::*, error::OreError, instruction::ClaimArgs, loaders::*, state::Proof};
use ore_utils::spl::transfer_signed;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::AccountDeserialize;

/// Claim distributes Ore from the treasury to a miner. Its responsibilies include:
/// 1. Decrement the miner's claimable balance.
/// 2. Transfer tokens from the treasury to the miner.
///
/// Safety requirements:
/// - Claim is a permissionless instruction and can be called by any user.
/// - Can only succeed if the claimed amount is less than or equal to the miner's claimable rewards.
/// - The provided beneficiary, token account, treasury, treasury token account, and token program must be valid.
pub fn process_claim<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = ClaimArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer, beneficiary_info, proof_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_token_account(beneficiary_info, None, &MINT_ADDRESS, true)?;
    load_proof(proof_info, signer.key, true)?;
    load_treasury(treasury_info, false)?;
    load_treasury_tokens(treasury_tokens_info, true)?;
    load_program(token_program, spl_token::id())?;

    // Update miner balance
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;

    if proof.balance < amount {
        return Err(OreError::ClaimTooLarge.into());
    }

    // Distribute tokens from treasury to beneficiary
    transfer_signed(
        treasury_info,
        treasury_tokens_info,
        beneficiary_info,
        token_program,
        amount,
        &[&[TREASURY, &[TREASURY_BUMP]]],
    )?;

    proof.balance = proof
        .balance
        .checked_sub(amount)
        .ok_or(OreError::ClaimTooLarge)?;

    Ok(())
}
