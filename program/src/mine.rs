use ore_api::prelude::*;
use solana_program::hash;
use steel::*;

/// Mine validates hashes and increments a miner's claimable balance.
pub fn process_mine(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Mine::try_from_bytes(data)?;

    // Load accounts.
    let clock = Clock::get()?;
    let t: i64 = clock.unix_timestamp;
    let [signer_info, config_info, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info.as_account_mut::<Config>(&ore_api::ID)?;
    let proof = proof_info
        .as_account_mut::<Proof>(&ore_api::ID)?
        .assert_mut_err(
            |p| p.miner == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?;

    // Compute the hash.
    let solution = hash::hashv(&[
        args.nonce.as_slice(),
        config.challenge.as_slice(),
        proof.authority.to_bytes().as_slice(),
    ]);

    // Update the best solution.
    if solution.to_bytes() < config.best_hash {
        config.best_hash = solution.to_bytes();
        config.best_proof = *proof_info.key;
    }

    // Update the proof.
    proof.last_hash = solution.to_bytes();
    proof.last_hash_at = t;
    proof.total_hashes += 1;

    Ok(())
}
