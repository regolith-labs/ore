use std::mem::size_of;

use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    keccak::hashv,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

use crate::{instruction::ProofArgs, loaders::*, state::Proof, utils::create_pda, PROOF};

pub fn process_proof<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let args = bytemuck::try_from_bytes::<ProofArgs>(data)
        .or(Err(ProgramError::InvalidInstructionData))?;

    // Account 1: Signer
    let signer = load_signer(next_account_info(accounts_iter)?)?;

    // Account 2: Proof
    let proof_info = load_uninitialized_pda(
        next_account_info(accounts_iter)?,
        &[PROOF, signer.key.as_ref(), &[args.bump]],
    )?;

    // Account 3: System program
    let system_program = load_account(next_account_info(accounts_iter)?, system_program::id())?;

    // Initialize proof
    create_pda(
        proof_info,
        &crate::id(),
        size_of::<Proof>(),
        &[PROOF, signer.key.as_ref(), &[args.bump]],
        system_program,
        signer,
    )?;
    let mut proof_data = proof_info.data.borrow_mut();
    let mut proof = bytemuck::try_from_bytes_mut::<Proof>(&mut proof_data).unwrap();
    proof.bump = args.bump as u64;
    proof.authority = *signer.key;
    proof.claimable_rewards = 0;
    proof.hash = hashv(&[&signer.key.to_bytes()]).into();
    proof.total_hashes = 0;
    proof.total_rewards = 0;

    Ok(())
}
