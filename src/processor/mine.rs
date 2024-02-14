use std::mem::size_of;

use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    keccak::{hashv, Hash as KeccakHash},
    program_error::ProgramError,
    pubkey::Pubkey,
    slot_hashes::SlotHash,
    sysvar::{self, Sysvar},
};

use crate::{
    instruction::MineArgs,
    loaders::*,
    state::{Bus, Proof, Treasury},
    EPOCH_DURATION,
};

pub fn process_mine<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args =
        bytemuck::try_from_bytes::<MineArgs>(data).or(Err(ProgramError::InvalidInstructionData))?;

    // Validate accounts
    let [signer, bus_info, proof_info, treasury_info, slot_hashes_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_bus(bus_info)?;
    load_proof(proof_info, signer.key)?;
    load_treasury(treasury_info)?;
    load_account(slot_hashes_info, sysvar::slot_hashes::id())?;

    // Validate epoch is active
    let clock = Clock::get().unwrap();
    let treasury_data = treasury_info.data.borrow();
    let treasury = bytemuck::try_from_bytes::<Treasury>(&treasury_data).unwrap();
    let epoch_end_at = treasury.epoch_start_at.saturating_add(EPOCH_DURATION);
    if !clock.unix_timestamp.lt(&epoch_end_at) {
        return Err(ProgramError::Custom(1));
    }

    // Validate provided hash
    let mut proof_data = proof_info.data.borrow_mut();
    let mut proof = bytemuck::try_from_bytes_mut::<Proof>(&mut proof_data).unwrap();
    validate_hash(
        proof.hash.into(),
        args.hash.into(),
        *signer.key,
        u64::from_le_bytes(args.nonce),
        treasury.difficulty.into(),
    )?;

    // Update claimable rewards
    let mut bus_data = bus_info.data.borrow_mut();
    let mut bus = bytemuck::try_from_bytes_mut::<Bus>(&mut bus_data).unwrap();
    if bus.available_rewards.lt(&treasury.reward_rate) {
        return Err(ProgramError::Custom(1));
    }
    bus.available_rewards = bus.available_rewards.saturating_sub(treasury.reward_rate);
    proof.claimable_rewards = proof.claimable_rewards.saturating_add(treasury.reward_rate);

    // Hash most recent slot hash into the next challenge to prevent pre-mining attacks
    let slot_hash_bytes = &slot_hashes_info.data.borrow()[0..size_of::<SlotHash>()];
    proof.hash = hashv(&[KeccakHash::from(args.hash).as_ref(), slot_hash_bytes]).into();

    // Update lifetime stats
    proof.total_hashes = proof.total_hashes.saturating_add(1);
    proof.total_rewards = proof.total_rewards.saturating_add(1);

    // TODO Log?

    Ok(())
}

pub(crate) fn validate_hash(
    current_hash: KeccakHash,
    hash: KeccakHash,
    signer: Pubkey,
    nonce: u64,
    difficulty: KeccakHash,
) -> Result<(), ProgramError> {
    // Validate hash correctness.
    let hash_ = hashv(&[
        current_hash.as_ref(),
        signer.as_ref(),
        nonce.to_be_bytes().as_slice(),
    ]);
    if !hash.eq(&hash_) {
        return Err(ProgramError::Custom(1));
    }

    // Validate hash difficulty.
    if !hash.le(&difficulty) {
        return Err(ProgramError::Custom(1));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use solana_program::{
        keccak::{hashv, Hash},
        pubkey::Pubkey,
    };

    use crate::validate_hash;

    #[test]
    fn test_validate_hash_pass() {
        let h1 = Hash::new_from_array([1; 32]);
        let signer = Pubkey::new_unique();
        let nonce = 10u64;
        let difficulty = Hash::new_from_array([255; 32]);
        let h2 = hashv(&[
            h1.to_bytes().as_slice(),
            signer.to_bytes().as_slice(),
            nonce.to_be_bytes().as_slice(),
        ]);
        let res = validate_hash(h1, h2, signer, nonce, difficulty);
        assert!(res.is_ok());
    }

    #[test]
    fn test_validate_hash_fail() {
        let h1 = Hash::new_from_array([1; 32]);
        let signer = Pubkey::new_unique();
        let nonce = 10u64;
        let difficulty = Hash::new_from_array([255; 32]);
        let h2 = Hash::new_from_array([2; 32]);
        let res = validate_hash(h1, h2, signer, nonce, difficulty);
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_hash_fail_difficulty() {
        let h1 = Hash::new_from_array([1; 32]);
        let signer = Pubkey::new_unique();
        let nonce = 10u64;
        let difficulty = Hash::new_from_array([0; 32]);
        let h2 = hashv(&[
            h1.to_bytes().as_slice(),
            signer.to_bytes().as_slice(),
            nonce.to_be_bytes().as_slice(),
        ]);
        let res = validate_hash(h1, h2, signer, nonce, difficulty);
        assert!(res.is_err());
    }
}
