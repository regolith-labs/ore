use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use steel::*;

/// Opens a new block.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Open::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);

    // Load accounts.
    let [signer_info, block_info, market_info, system_program, ore_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    block_info
        .is_empty()? // Account has not been initialized
        .is_writable()? // Account is writable
        .has_seeds(&[BLOCK, &id.to_le_bytes()], &ore_api::ID)?; // Account has correct seeds
    market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.block_id < id)?; // Only allow opening blocks in forward bias
    system_program.is_program(&system_program::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Create block account.
    create_program_account::<Block>(
        block_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[BLOCK, &id.to_le_bytes()],
    )?;
    let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    block.id = id;
    block.opener = *signer_info.key;
    block.reward = calculate_reward(block.id);
    block.best_hash = [0; 32];
    block.best_hash_miner = Pubkey::default();
    block.start_slot = u64::MAX; // Set by reset
    block.end_slot = u64::MAX; // Set by reset
    block.slot_hash = [0; 32]; // Set by mine
    block.total_hashpower = 0;

    // Emit event.
    // program_log(
    //     id,
    //     &[block_info.clone(), ore_program.clone()],
    //     &OpenEvent {
    //         disc: OreEvent::Open as u64,
    //         id,
    //         start_slot,
    //         signer: *signer_info.key,
    //         reward_config: block.reward,
    //         // liquidity_base: market.base.liquidity() as u64,
    //         // liquidity_quote: market.quote.liquidity() as u64,
    //         ts: clock.unix_timestamp,
    //     }
    //     .to_bytes(),
    // )?;

    Ok(())
}

fn calculate_reward(block_id: u64) -> u64 {
    // Generate noise.
    let noise_seed = block_id.to_le_bytes();
    let noise = hash(&noise_seed);

    // Extract the first byte (0 to 255).
    let byte_value = noise[0];

    // Map to 1-10 using integer division
    let n = (byte_value / 25) + 1;

    // Ensure the value doesn't exceed 10
    let n = n.min(10);

    n as u64 * ONE_ORE
}

#[test]
fn test_lode_rewards() {
    for i in 0u64..1000 {
        let lode_reward = ONE_ORE * calculate_reward(i) as u64;
        let target_block_reward = ONE_ORE * 10;
        let expected_hashes_per_block = HASHPOWER_LIQUIDITY / 2;
        let expected_qualifying_hashes =
            expected_hashes_per_block / 2u64.pow(NUGGET_DIFFICULTY as u32);
        let difficulty_reward = (target_block_reward - lode_reward) / expected_qualifying_hashes;
        println!("{}: {} {}", i, lode_reward, difficulty_reward);
    }
    // assert!(false);
}
