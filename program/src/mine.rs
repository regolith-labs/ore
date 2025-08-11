use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use solana_program::pubkey;
use steel::*;

const AUTHORIZED_ACCOUNTS: [Pubkey; 2] = [
    pubkey!("pqspJ298ryBjazPAr95J9sULCVpZe3HbZTWkbC1zrkS"),
    pubkey!("6B9PjpHfbhPcSakS5UQ7ZctgbPujfsryVRpDecskGLiz"),
];

/// Mine a block.
pub fn process_mine(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Mine::try_from_bytes(data)?;
    let nonce = u64::from_le_bytes(args.nonce);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, miner_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.end_slot)? // Block has stopped trading
        .assert_mut(|b| clock.slot < b.end_slot + MINING_WINDOW)? // Give 1500 slots to submit hashes
        .assert_mut(|b| b.slot_hash != [0; 32])?; // Slot hash is set
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)? // Account belongs to authority
        .assert_mut(|m| m.block_id == block.id)? // Only allow miner to submit hashes for their current block
        .assert_mut(|m| m.hashpower > nonce)?; // Only allow miner to submit nonces for their hashpower range

    // Check if the signer is authorized.
    if !AUTHORIZED_ACCOUNTS.contains(signer_info.key) {
        return Err(ProgramError::InvalidAccountData);
    }

    // Generate secure hash with provided nonce.
    let mut seed = [0u8; 112];
    seed[..8].copy_from_slice(&block.id.to_le_bytes());
    seed[8..40].copy_from_slice(&block.slot_hash);
    seed[40..72].copy_from_slice(&miner.authority.to_bytes());
    seed[72..104].copy_from_slice(&miner.seed);
    seed[104..].copy_from_slice(&nonce.to_le_bytes());
    let h = hash(&seed);

    // If hash is best hash, update best hash.
    if h < block.best_hash {
        block.best_hash = h;
        block.best_hash_miner = miner.authority;
    }

    // Only allow miners to submit 1 hash per block.
    miner.hashpower = 0;

    Ok(())
}
