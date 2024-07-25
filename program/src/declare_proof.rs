use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16},
};


/// DeclareProof is used by other instructions in the same transaction.
/// - Other instructions will use transaction introspection to ensure they
/// only process the declared proof.
/// - Other instructions that need to get the declared proof for validation
/// will only look at the first and second instructions in the transaction
///
/// Safety requirements:
/// - No safety requirements are required in this instruction to keep cu's as
/// low as possible. Other instructions that use the declared proof handle
/// validation via the loader.
/// - Only one account should be provided.
pub fn process_declare_proof<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    let [_proof_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ok(())
}

/// Errors if:
/// - Fails to find and parse the declared proof pubkey in the second instruction
/// of the transaction
pub fn find_and_parse_declared_proof(data: &[u8]) -> Result<Pubkey, SanitizeError> {
    // start the current byte index at 0
    let mut current = 0;
    let num_instructions = read_u16(&mut current, data)?;

    if num_instructions < 2 {
        // Not enough instructions in this transaction.
        // The declare_proof instruction must be included with 
        // the other instruction in the same transaction.
        return Err(SanitizeError::IndexOutOfBounds)
    }

    // The first ix should be a set compute budget ix
    // The second ix should be a declare_proof ix
    // The starting index is the second ix
    let index = 1;

    current += index * 2;
    let start = read_u16(&mut current, data)?;

    current = start as usize;

    current += 2; // skip the accounts length variable
    current += 1; // skip the meta_byte

    // The only account provided should be the proof pubkey
    let proof_pubkey = read_pubkey(&mut current, data)?;

    // There shouldn't be any more accounts,
    // this should be the Ore program ID
    let program_pubkey = read_pubkey(&mut current, data)?;
    if program_pubkey.eq(&ore_api::ID) {
        return Ok(proof_pubkey);
    } else {
        return Err(SanitizeError::InvalidValue);
    }
}

