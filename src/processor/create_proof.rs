use std::mem::size_of;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, keccak::hashv,
    program_error::ProgramError, pubkey::Pubkey, system_program,
};

use crate::{
    instruction::CreateProofArgs,
    loaders::*,
    state::{Discriminator, Proof},
    utils::create_pda,
    PROOF,
};

pub fn process_create_proof<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = CreateProofArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_uninitialized_pda(proof_info, &[PROOF, signer.key.as_ref(), &[args.bump]])?;
    load_program(system_program, system_program::id())?;

    // Initialize proof
    create_pda(
        proof_info,
        &crate::id(),
        8 + size_of::<Proof>(),
        &[PROOF, signer.key.as_ref(), &[args.bump]],
        system_program,
        signer,
    )?;
    let mut proof_data = proof_info.data.borrow_mut();
    proof_data[0] = Proof::discriminator() as u8;
    let mut proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.bump = args.bump as u64;
    proof.authority = *signer.key;
    proof.claimable_rewards = 0;
    proof.hash = hashv(&[&signer.key.to_bytes()]).into();
    proof.total_hashes = 0;
    proof.total_rewards = 0;

    Ok(())
}
