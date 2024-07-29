use ore_api::consts::COMPUTE_BUDGET_PROGRAM_ID;
use solana_program::{
    pubkey,
    pubkey::Pubkey,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16},
};

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

    // Iterate through the transaction instructions
    for i in 0..num_instructions as usize {
        // Get byte counter
        curr = pc + i * 2;
        curr = read_u16(&mut curr, data)? as usize;

        // Read the instruction program id
        let num_accounts = read_u16(&mut curr, data)? as usize;
        curr += num_accounts * 33;
        let program_id = read_pubkey(&mut curr, data)?;

        // Introspect on the first non compute budget instruction
        if program_id.ne(&COMPUTE_BUDGET_PROGRAM_ID) {
            // Read address from ix data
            curr += 2;
            let address = read_pubkey(&mut curr, data)?;
            return Ok(Some(address));
        }
    }

    Ok(None)
}
