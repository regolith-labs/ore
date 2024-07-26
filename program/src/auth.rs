use ore_api::instruction::OreInstruction;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey,
    pubkey::Pubkey,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16, read_u8},
};

/// DeclareProof is used by other instructions in the same transaction.
/// - Other instructions will use transaction introspection to ensure they
/// only process the declared proof.
/// - Other instructions will use find_and_parse_declared_proof with the
/// introspection data
///
/// Safety requirements:
/// - No safety requirements are required in this instruction to keep cu's as
/// low as possible. Other instructions that use the declared proof handle
/// validation via the loader.
/// - Only one account should be provided.
pub fn process_auth<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    let [_proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ok(())
}

const COMPUTE_BUDGET_PROGRAM_ID: Pubkey = pubkey!("ComputeBudget111111111111111111111111111111");

/// Require that only the declared proof can be processed in this transaction.
///
/// The intent here is to disincentivize sybil. As long as a user can fit multiple hashes in a single
/// transaction, there is a financial incentive to sybil multiple keypairs and pack as many hashes
/// as possible into each transaction to minimize fee / hash.
///
/// If each transaction is limited to one hash only, then a user will minimize their fee / hash
/// by allocating all their hashpower to finding the single most difficult hash they can.
///
/// Errors if:
/// - Fails to find and parse the declared proof pubkey in the second instruction
/// of the transaction
pub fn authenticate(data: &[u8]) -> Result<Option<Pubkey>, SanitizeError> {
    // Start the current byte index at 0
    let mut curr = 0;
    let num_instructions = read_u16(&mut curr, data)?;
    let pc = curr;
    for i in 0..num_instructions as usize {
        curr = pc + i * 2;
        curr = read_u16(&mut curr, data)? as usize;
        let num_accounts = read_u16(&mut curr, data)? as usize;
        let mut ac = curr;
        curr += num_accounts * 33;
        let program_id = read_pubkey(&mut curr, data)?;
        if program_id.eq(&ore_api::ID) {
            curr += 2;
            if let Ok(ix) = OreInstruction::try_from(read_u8(&mut curr, data)?) {
                // Validate ix is an auth
                if ix.ne(&OreInstruction::Auth) {
                    return Ok(None);
                }
                // Valid num accounts is correct
                if num_accounts.ne(&1) {
                    return Ok(None);
                }
                // Return provided pubkey
                let address = read_pubkey(&mut ac, data)?;
                return Ok(Some(address));
            } else {
                // Otherwise return
                return Ok(None);
            }
        }
    }

    Ok(None)
}
