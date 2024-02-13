use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    keccak::{hashv, Hash},
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub fn process_mine<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // TODO
    Ok(())
}

pub(crate) fn validate_hash(
    current_hash: Hash,
    hash: Hash,
    signer: Pubkey,
    nonce: u64,
    difficulty: Hash,
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
