use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};


/// DeclareProof is used by other instructions in the same transaction.
/// - Other instructions will use transaction introspection to ensure they
/// only process the declared proof.
/// - Other instructions that need to validate the declared proof will only 
/// look at the first and second instructions in the transaction
///
/// Safety requirements:
/// - No safety requirements are required in this instruction to keep cu's as
/// low as possible. Other instructions that use the declared proof handle
/// validation via the loader.
pub fn process_declare_proof<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Ensure one account info is provided
    // Validation of this proof is handled by the mine ix
    let [_proof_info] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ok(())
}

