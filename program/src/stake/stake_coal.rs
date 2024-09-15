use coal_api::{consts::*, instruction::StakeArgs, loaders::*, state::Proof};
use coal_utils::spl::transfer;
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, sysvar::Sysvar,
};

use crate::utils::AccountDeserialize;

/// Stake deposits ORE into a proof account to earn multiplier.
pub fn process_stake_coal<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = StakeArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer, proof_info, sender_info, treasury_tokens_info, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_coal_proof(proof_info, signer.key, true)?;
    load_token_account(sender_info, Some(signer.key), &COAL_MINT_ADDRESS, true)?;
    load_coal_treasury_tokens(treasury_tokens_info, true)?;
    load_program(token_program, spl_token::id())?;

    // Update the proof balance.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.balance = proof.balance.checked_add(amount).unwrap();

    // Update deposit timestamp.
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    proof.last_stake_at = clock.unix_timestamp;

    // Transfer tokens from signer to treasury.
    transfer(
        signer,
        sender_info,
        treasury_tokens_info,
        token_program,
        amount,
    )?;

    Ok(())
}