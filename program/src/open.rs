use std::mem::size_of;

use ore_api::prelude::*;
use solana_program::{keccak::hashv, slot_hashes::SlotHash};
use steel::*;

/// Open creates a new proof account to track a miner's state.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Open::try_from_bytes(data)?;

    // Load accounts.
    let [signer_info, miner_info, payer_info, proof_info, system_program, slot_hashes_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    payer_info.is_signer()?;
    proof_info.is_empty()?.is_writable()?.has_seeds(
        &[PROOF, signer_info.key.as_ref()],
        args.bump,
        &ore_api::ID,
    )?;
    system_program.is_program(&system_program::ID)?;
    slot_hashes_info.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Initialize proof.
    create_account::<Proof>(
        proof_info,
        &ore_api::ID,
        &[PROOF, signer_info.key.as_ref(), &[args.bump]],
        system_program,
        payer_info,
    )?;
    let clock = Clock::get()?;
    let proof = proof_info.to_account_mut::<Proof>(&ore_api::ID)?;
    proof.authority = *signer_info.key;
    proof.balance = 0;
    proof.challenge = hashv(&[
        signer_info.key.as_ref(),
        &slot_hashes_info.data.borrow()[0..size_of::<SlotHash>()],
    ])
    .0;
    proof.last_hash = [0; 32];
    proof.last_hash_at = clock.unix_timestamp;
    proof.last_stake_at = clock.unix_timestamp;
    proof.miner = *miner_info.key;
    proof.total_hashes = 0;
    proof.total_rewards = 0;

    Ok(())
}
