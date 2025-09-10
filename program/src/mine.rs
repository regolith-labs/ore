use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use solana_program::log::sol_log;
use steel::*;

use crate::whitelist::AUTHORIZED_ACCOUNTS;

/// Mine a block.
pub fn process_mine(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Mine::try_from_bytes(data)?;
    let nonce = u64::from_le_bytes(args.nonce);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, market_info, miner_info, ore_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.end_slot)? // Block has stopped trading
        .assert_mut(|b| clock.slot < b.end_slot + MINING_WINDOW)? // Give 1500 slots to submit hashes
        .assert_mut(|b| b.slot_hash != [0; 32])?; // Slot hash is set
    market_info.as_account::<Market>(&ore_api::ID)?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key || m.executor == *signer_info.key)? // Account belongs to authority
        .assert_mut(|m| m.block_id == block.id)? // Only allow miner to submit hashes for their current block
        .assert_mut(|m| m.hashpower > nonce)?; // Only allow miner to submit nonces for their hashpower range
    ore_program.is_program(&ore_api::ID)?;

    // Check if the signer is authorized.
    if !AUTHORIZED_ACCOUNTS.contains(signer_info.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    sol_log(&format!("Authorized account: {}", signer_info.key));

    // Generate secure hash with provided nonce.
    let mut seed = [0u8; 112];
    seed[..8].copy_from_slice(&block.id.to_le_bytes());
    seed[8..40].copy_from_slice(&block.slot_hash);
    seed[40..72].copy_from_slice(&miner.authority.to_bytes());
    seed[72..104].copy_from_slice(&miner.seed);
    seed[104..].copy_from_slice(&nonce.to_le_bytes());
    let h = hash(&seed);

    sol_log(&format!("Slot hash: {:?}", block.slot_hash));
    sol_log(&format!("Authority: {:?}", miner.authority));
    sol_log(&format!("Nonce: {:?}", nonce));
    sol_log(&format!("Seed: {:?}", miner.seed));
    sol_log(&format!("Hash: {:?}", h));

    // If hash is best hash, update best hash.
    if h < block.best_hash {
        block.best_hash = h;
        block.best_hash_miner = miner.authority;
    }

    // Emit event.

    program_log(
        &[market_info.clone(), ore_program.clone()],
        &MineEvent {
            disc: 2,
            authority: *signer_info.key,
            block_id: block.id,
            nonce,
            hashpower: miner.hashpower,
            is_best: (block.best_hash_miner == miner.authority) as u64,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

    // Only allow miners to submit 1 hash per block.
    miner.hashpower = 0;

    Ok(())
}
