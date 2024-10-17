use steel::*;

/// Stake deposits ORE into a proof account to earn multiplier.
pub fn process_stake(_accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    panic!("This instruction has been deprecated. Please stake with the boost program instead.");
}
