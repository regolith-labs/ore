use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use steel::*;

/// Mine a block.
pub fn process_mine(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Mine::try_from_bytes(data)?;
    let nonce = u64::from_le_bytes(args.nonce);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, authority_info, block_info, miner_info, ore_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    authority_info.is_writable()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.end_slot)? // Block has stopped trading
        .assert_mut(|b| clock.slot < b.end_slot + MINING_WINDOW)? // Give 1500 slots to submit hashes
        .assert_mut(|b| b.slot_hash != [0; 32])?; // Slot hash is set
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *authority_info.key)? // Account belongs to authority
        .assert_mut(|m| m.block_id <= block.id)?; // Only allow miner to submit hashes in forward bias
    ore_program.is_program(&ore_api::ID)?;

    // Reset miner hash if mining new block.
    if miner.block_id != block.id {
        let mut args = [0u8; 104];
        args[..8].copy_from_slice(&block.id.to_le_bytes());
        args[8..40].copy_from_slice(&block.slot_hash);
        args[40..72].copy_from_slice(&miner.authority.to_bytes());
        args[72..].copy_from_slice(&miner.seed);
        miner.hash = hash(&args);
        miner.block_id = block.id;
    }

    // Generate secure hash with provided nonce.
    let mut seed = [0u8; 40];
    seed[..8].copy_from_slice(&miner.hash.as_ref());
    seed[8..40].copy_from_slice(&nonce.to_le_bytes());
    let h = hash(&seed);

    // If hash is best hash, update best hash.
    if h < block.best_hash {
        block.best_hash = h;
        block.best_hash_miner = miner.authority;
    }

    // Emit event.
    // program_log(
    //     block.id,
    //     &[block_info.clone(), ore_program.clone()],
    //     &MineEvent {
    //         disc: OreEvent::Mine as u64,
    //         authority: miner.authority,
    //         block_id: block.id,
    //         deployed: amount,
    //         total_deployed: block.total_deployed,
    //         remaining_commitment: 0,
    //         ts: clock.unix_timestamp,
    //     }
    //     .to_bytes(),
    // )?;

    Ok(())
}
