use std::mem::size_of;

use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, keccak::hashv,
    program_error::ProgramError, pubkey::Pubkey, slot_hashes::SlotHash, system_program, sysvar,
};

use crate::{
    instruction::RegisterArgs,
    loaders::*,
    state::Proof,
    utils::AccountDeserialize,
    utils::{create_pda, Discriminator},
    PROOF,
};

// TODO Create a stake account (token account)
// TODO Need to keep claimable balance (from treasury) separate from staking balance (from stake account)
// TODO Multiplier calculations should account for both

/// Register generates a new hash chain for a prospective miner. Its responsibilities include:
/// 1. Initialize a new proof account.
/// 2. Generate an initial hash from the signer's key.
///
/// Safety requirements:
/// - Register is a permissionless instruction and can be invoked by any singer.
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
    let [signer, proof_info, system_program, slot_hashes_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_uninitialized_pda(
        proof_info,
        &[PROOF, signer.key.as_ref()],
        args.bump,
        &crate::id(),
    )?;
    load_program(system_program, system_program::id())?;
    load_sysvar(slot_hashes_info, sysvar::slot_hashes::id())?;

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
    proof.balance = 0;
    proof.hash = hashv(&[
        signer.key.as_ref(),
        &slot_hashes_info.data.borrow()[0..size_of::<SlotHash>()],
    ])
    .0;
    proof.last_hash_at = 0;
    proof.total_hashes = 0;
    proof.total_rewards = 0;

    Ok(())
}
