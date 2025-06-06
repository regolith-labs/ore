use ore_api::prelude::*;
use solana_program::{keccak, slot_hashes::SlotHashes};
use steel::*;

/// Mine a block.
pub fn process_mine(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Mine::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, market_info, miner_info, mint_info, sender_info, system_program, token_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot)?
        .assert_mut(|b| clock.slot < b.start_slot + 1500)?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.id == block.id)?;
    mint_info.has_address(&market.base.mint)?.as_mint()?;
    sender_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_info.key)?
        .assert(|t| t.amount() >= amount)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Load miner account.
    let miner = if miner_info.data_is_empty() {
        create_program_account::<Miner>(
            miner_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[MINER, &signer_info.key.to_bytes()],
        )?;
        let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
        miner.authority = *signer_info.key;
        miner.block_id = 0;
        miner.hash = [0; 32];
        miner.total_hashes = 0;
        miner.total_rewards = 0;
        miner
    } else {
        miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.authority == *signer_info.key)?
    };

    // Update miner stats.
    miner.total_hashes += amount;

    // Burn hash tokens.
    burn(sender_info, mint_info, signer_info, token_program, amount)?;

    // Set block slot hash.
    if block.slot_hash == [0; 32] {
        let slot_hashes =
            bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
        let Some(slot_hash) = slot_hashes.get(&block.start_slot) else {
            // If mine is not called within 2.5 minutes of the block starting,
            // then the slot hash will be unavailable and secure hashes cannot be generated.
            return Ok(());
        };
        block.slot_hash = slot_hash.to_bytes();
    }

    // Reset miner hash if mining new block.
    if miner.block_id != block.id {
        miner.block_id = block.id;
        miner.hash =
            keccak::hashv(&[block.slot_hash.as_ref(), miner.authority.as_ref()]).to_bytes();
    }

    // Mine.
    for _ in 0..amount {
        miner.hash = keccak::hashv(&[miner.hash.as_ref()]).to_bytes();
        if miner.hash < block.best_hash {
            block.best_hash = miner.hash;
            block.best_miner = miner.authority;
        }
    }

    Ok(())
}
