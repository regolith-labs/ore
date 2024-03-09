use std::mem::size_of;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, keccak::hashv,
    program_error::ProgramError, pubkey::Pubkey, system_program,
};

use crate::{
    instruction::RegisterArgs,
    loaders::*,
    state::Proof,
    utils::AccountDeserialize,
    utils::{create_pda, Discriminator},
    PROOF,
};

/// Register generates a new hash chain for a prospective miner. It has 2 responsibilities:
/// 1. Initializes a new proof account.
/// 2. Generates an initial hash for the miner from the signer's key.
///
/// Safety requirements:
/// - Register is a permissionless instruction and can be called by anyone.
/// - Can only succeed if the provided proof acount PDA is valid (associated with the signer).
/// - Can only succeed once per signer.
/// - The provided system program must be valid.
pub fn process_register<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = RegisterArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_uninitialized_pda(
        proof_info,
        &[PROOF, signer.key.as_ref(), &[args.bump]],
        &crate::id(),
    )?;
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
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.authority = *signer.key;
    proof.claimable_rewards = 0;
    proof.hash = hashv(&[signer.key.as_ref()]).into();
    proof.total_hashes = 0;
    proof.total_rewards = 0;

    Ok(())
}
