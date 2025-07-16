use ore_api::{prelude::*, sdk::program_log};
use solana_nostd_keccak::hash;
use solana_program::{log::sol_log, slot_hashes::SlotHashes};
use steel::*;

/// Mine a block.
pub fn process_mine(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Mine::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, authority_info, block_info, market_info, miner_info, mint_info, recipient_info, treasury_info, system_program, token_program, ore_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    authority_info.is_writable()?;
    let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    // .assert_mut(|b| clock.slot >= b.start_slot)?
    // .assert_mut(|b| clock.slot < b.start_slot + 1500)?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.block_id == block.id)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *authority_info.key)?;
    recipient_info
        .is_writable()?
        .as_associated_token_account(&miner.authority, &MINT_ADDRESS)?;
    treasury_info.has_address(&TREASURY_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Burn hash tokens.
    // burn_signed(
    //     commitment_info,
    //     mint_hash_info,
    //     block_info,
    //     token_program,
    //     amount,
    //     &[BLOCK, &block.id.to_le_bytes()],
    // )?;

    // Set block slot hash.
    // if block.slot_hash == [0; 32] {
    //     let slot_hashes =
    //         bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
    //     let Some(slot_hash) = slot_hashes.get(&block.start_slot) else {
    //         // If mine is not called within ~2.5 minutes of the block starting,
    //         // then the slot hash will be unavailable and secure hashes cannot be generated.
    //         return Ok(());
    //     };
    //     block.slot_hash = slot_hash.to_bytes();
    // }

    // Reset miner hash if mining new block.
    // if miner.block_id != block.id {
    //     let mut args = [0u8; 104];
    //     args[..8].copy_from_slice(&block.id.to_le_bytes());
    //     args[8..40].copy_from_slice(&block.slot_hash);
    //     args[40..72].copy_from_slice(&miner.authority.to_bytes());
    //     args[72..].copy_from_slice(&miner.seed);
    //     miner.hash = hash(&args);
    //     miner.block_id = block.id;
    // }

    // Mine and accumulate rewards.
    // let mut nugget_reward = 0;
    // for _ in 0..amount {
    //     // Update stats
    //     // block.total_deployed += 1;

    //     // Generate hash.
    //     miner.hash = hash(miner.hash.as_ref());

    //     // Score and increment rewards.
    //     let score = difficulty(miner.hash) as u64;
    //     if score >= block.reward.nugget_threshold {
    //         nugget_reward += block.reward.nugget_reward;
    //     }

    //     // If hash is best hash, update best hash.
    //     if miner.hash < block.reward.lode_hash {
    //         block.reward.lode_hash = miner.hash;
    //         block.reward.lode_authority = miner.authority;
    //     }
    // }

    // Log mint.
    // let ore_mint = mint_ore_info.as_mint()?;
    // let mint_authority = ore_mint.mint_authority();
    // sol_log(format!("mint_authority: {:?}", mint_authority).as_str());
    // sol_log(format!("treasury: {:?}", treasury_info.key).as_str());

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

/// Returns the number of leading zeros on a 32 byte buffer.
pub fn difficulty(hash: [u8; 32]) -> u32 {
    let mut count = 0;
    for &byte in &hash {
        let lz = byte.leading_zeros();
        count += lz;
        if lz < 8 {
            break;
        }
    }
    count
}
