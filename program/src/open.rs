use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use steel::*;

/// Opens a new block.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Open::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, system_program, ore_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    block_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BLOCK, &id.to_le_bytes()], &ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // TODO

    // Error out if start slot is within the current period.
    let start_slot = id * 1500;
    let current_block = clock.slot / 1500;
    let current_period_start = current_block * 1500;
    let current_period_end = current_period_start + 1500;
    if start_slot < current_period_end {
        return Err(ProgramError::InvalidArgument);
    }

    // Initialize block.
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
    block.reward = ONE_ORE * generate_lode(block.id) as u64;
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

fn generate_lode(block_id: u64) -> u8 {
    // Generate noise.
    let noise_seed = block_id.to_le_bytes();
    let noise = hash(&noise_seed);

    // Extract the first byte (0 to 255).
    let byte_value = noise[0];

    // Map to 1-10 using integer division
    let reward = (byte_value / 25) + 1;

    // Ensure the value doesn't exceed 10
    if reward > 10 {
        10
    } else {
        reward
    }
}

#[test]
fn test_lode_rewards() {
    for i in 0u64..1000 {
        let lode_reward = ONE_ORE * generate_lode(i) as u64;
        let target_block_reward = ONE_ORE * 10;
        let expected_hashes_per_block = HASHPOWER_LIQUIDITY / 2;
        let expected_qualifying_hashes =
            expected_hashes_per_block / 2u64.pow(NUGGET_DIFFICULTY as u32);
        let difficulty_reward = (target_block_reward - lode_reward) / expected_qualifying_hashes;
        println!("{}: {} {}", i, lode_reward, difficulty_reward);
    }
    // assert!(false);
}
