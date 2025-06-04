use ore_api::prelude::*;
use solana_program::keccak;
use steel::*;

/// Opens a new block for hashpower trading.
pub fn process_mine(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Mine::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, miner_info, system_program, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot)?
        .assert_mut(|b| clock.slot < b.start_slot + 1500)?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // TODO Open miner account if it doesn't exist.
    // TODO Burn hash tokens

    // Reset miner hash if mining new block.
    if miner.block_id != block.id {
        miner.block_id = block.id;
        miner.hash =
            keccak::hashv(&[block.slot_hash.as_ref(), miner.authority.as_ref()]).to_bytes();
    }

    for _ in 0..amount {
        miner.hash = keccak::hashv(&[miner.hash.as_ref()]).to_bytes();
        if miner.hash < block.best_hash {
            block.best_hash = miner.hash;
            block.best_miner = miner.authority;
        }
    }

    // Update miner stats.
    miner.total_hashes += amount;

    Ok(())
}
