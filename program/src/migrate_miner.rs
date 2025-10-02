use ore_api::prelude::*;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Pays out the winners and block reward.
pub fn process_migrate_miner(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // TODO Migrate miner account.
    // TODO Move refund_sol into rewards_sol.

    Ok(())
}
