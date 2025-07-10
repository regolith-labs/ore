use ore_api::{prelude::*, sdk::program_log};
use solana_nostd_keccak::hash;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Mine a block.
pub fn process_mine(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Mine::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, authority_info, block_info, commitment_info, market_info, miner_info, mint_hash_info, mint_ore_info, permit_info, recipient_info, treasury_info, system_program, token_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    authority_info.is_writable()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot)?
        .assert_mut(|b| clock.slot < b.start_slot + 1500)?;
    commitment_info
        .is_writable()?
        .has_address(&commitment_pda(block.id).0)?
        .as_token_account()?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.id == block.id)?;
    mint_hash_info.has_address(&market.base.mint)?.as_mint()?;
    mint_ore_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *authority_info.key)?;
    let permit = permit_info
        .as_account_mut::<Permit>(&ore_api::ID)?
        .assert_mut(|p| p.authority == miner.authority)?
        .assert_mut(|p| p.block_id == block.id)?
        .assert_mut(|p| p.executor == *signer_info.key || p.executor == Pubkey::default())?;
    recipient_info
        .is_writable()?
        .as_associated_token_account(&miner.authority, &MINT_ADDRESS)?;
    treasury_info.has_address(&TREASURY_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Reduce permit amount.
    let amount = permit.commitment.min(amount);
    permit.commitment -= amount;

    // Pay executor fee.
    if permit.fee > 0 {
        permit_info.send(permit.fee * amount, signer_info);
    }

    // Close permit account, if empty.
    if permit.commitment == 0 {
        permit_info.close(authority_info)?;
    }

    // Burn hash tokens.
    burn_signed(
        commitment_info,
        mint_hash_info,
        block_info,
        token_program,
        amount,
        &[BLOCK, &block.id.to_le_bytes()],
    )?;

    // Set block slot hash.
    if block.slot_hash == [0; 32] {
        let slot_hashes =
            bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
        let Some(slot_hash) = slot_hashes.get(&block.start_slot) else {
            // If mine is not called within ~2.5 minutes of the block starting,
            // then the slot hash will be unavailable and secure hashes cannot be generated.
            return Ok(());
        };
        block.slot_hash = slot_hash.to_bytes();
    }

    // Reset miner hash if mining new block.
    if miner.block_id != block.id {
        let mut args = [0u8; 128];
        args[..32].copy_from_slice(&block.id.to_le_bytes());
        args[32..64].copy_from_slice(&block.slot_hash);
        args[64..96].copy_from_slice(&permit.authority.to_bytes());
        args[96..].copy_from_slice(&permit.seed);
        miner.hash = hash(&args);
        miner.block_id = block.id;
    }

    // Mine and accumulate rewards.
    let mut nugget_reward = 0;
    for _ in 0..amount {
        // Update stats
        block.total_deployed += 1;
        miner.total_deployed += 1;

        // Generate hash.
        miner.hash = hash(miner.hash.as_ref());

        // Score and increment rewards.
        let score = difficulty(miner.hash) as u64;
        if score >= block.reward.nugget_threshold {
            nugget_reward += block.reward.nugget_reward;
        }

        // If hash is best hash, update best hash.
        if miner.hash < block.reward.lode_hash {
            block.reward.lode_hash = miner.hash;
            block.reward.lode_authority = miner.authority;
        }
    }

    // Payout ORE.
    if nugget_reward > 0 {
        // Limit payout to supply cap.
        let ore_mint = mint_ore_info.as_mint()?;
        let max_reward = MAX_SUPPLY.saturating_sub(ore_mint.supply());
        let reward_amount = nugget_reward.min(max_reward);

        // Update stats.
        block.total_rewards += reward_amount;
        miner.total_rewards += reward_amount;

        // Mint to recipient.
        mint_to_signed(
            mint_ore_info,
            recipient_info,
            treasury_info,
            token_program,
            reward_amount,
            &[TREASURY],
        )?;

        // Emit event.
        program_log(
            block.id,
            block_info.clone(),
            &RewardEvent {
                disc: OreEvent::Reward as u64,
                amount: reward_amount,
                authority: miner.authority,
                block_id: block.id,
                rewards_type: RewardsType::Nugget as u64,
                ts: clock.unix_timestamp,
            }
            .to_bytes(),
        )?;
    }

    // Emit event.
    program_log(
        block.id,
        block_info.clone(),
        &MineEvent {
            disc: OreEvent::Mine as u64,
            authority: miner.authority,
            block_id: block.id,
            deployed: amount,
            total_deployed: block.total_deployed,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

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
