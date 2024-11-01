use steel::*;

/// Upgrade allows a user to migrate a v1 token to a v2 token at a 1:1 exchange rate.
pub fn process_upgrade(_accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    panic!("This instruction has been deprecated. v1 tokens are no longer eligable to upgrade.");
}
