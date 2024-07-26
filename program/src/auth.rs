use ore_api::instruction::OreInstruction;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
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
pub fn process_auth<'a, 'info>(accounts: &'a [AccountInfo<'info>], _data: &[u8]) -> ProgramResult {
    let [_proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    Ok(())
}

/// Get the authenticated pubkey.
///
/// The intent here is to disincentivize sybil. If a user can fit multiple hashes into a single
/// transaction, there is a financial incentive to sybil multiple keypairs and pack as many hashes
/// as possible into each transaction to minimize fee / hash.
///
/// If each transaction is limited to one hash only, then a user will minimize their fee / hash
/// by allocating all their hashpower to finding the single most difficult hash they can.
///
/// We solve this by "authenticating" the proof account on every mine instruction. That is,
/// every transaction with a `mine` instruction needs to include an `auth` instruction that
/// specifies the proof account that will be used. The `auth` instruction must be first ORE
/// instruction in the transaction. The `mine` instruction should error out if the provided proof
/// account doesn't match the authenticated address.
///
/// Errors if:
/// - Fails to find and parse an authentication address.
pub fn authenticate(data: &[u8]) -> Result<Option<Pubkey>, SanitizeError> {
    // Start the current byte index at 0
    let mut curr = 0;
    let num_instructions = read_u16(&mut curr, data)?;
    let pc = curr;

    // Iterate through the top-level instructions
    for i in 0..num_instructions as usize {
        // Get byte counter
        curr = pc + i * 2;
        curr = read_u16(&mut curr, data)? as usize;

        // Read num accounts on this ix
        let num_accounts = read_u16(&mut curr, data)? as usize;

        // Hold a pointer to the first account in the accounts array
        let mut ac = curr + 1;

        // Read the instruction program id
        curr += num_accounts * 33;
        let program_id = read_pubkey(&mut curr, data)?;

        // We only need to introspect on the first ore ix
        if program_id.eq(&ore_api::ID) {
            curr += 2;

            // Parse the instruction data
            if let Ok(ix) = OreInstruction::try_from(read_u8(&mut curr, data)?) {
                // Return immediately if the ix is not an auth
                if ix.ne(&OreInstruction::Auth) {
                    return Ok(None);
                }

                // Valid the num accounts is expected
                if num_accounts.ne(&1) {
                    return Ok(None);
                }

                // Return the address of the authenticated account
                let address = read_pubkey(&mut ac, data)?;
                return Ok(Some(address));
            }

            // Return if instruction data can't be parsed
            return Ok(None);
        }
    }

    Ok(None)
}
