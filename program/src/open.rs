use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use solana_program::program_pack::Pack;
use spl_token_2022::instruction::AuthorityType;
use steel::*;

/// Opens a new block.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Open::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, collateral_info, commitment_info, market_info, mint_base_info, mint_quote_info, sender_info, treasury_info, vault_base_info, vault_quote_info, system_program, token_program, associated_token_program, rent_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    block_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BLOCK, &id.to_le_bytes()], &ore_api::ID)?;
    market_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[MARKET, &id.to_le_bytes()], &ore_api::ID)?;
    mint_base_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[MINT, &id.to_le_bytes()], &ore_api::ID)?;
    mint_quote_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    sender_info
        .is_writable()?
        .as_associated_token_account(&signer_info.key, &mint_quote_info.key)?
        .assert(|t| t.amount() >= OPEN_FEE)?;
    treasury_info.has_address(&TREASURY_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;
    rent_sysvar.is_sysvar(&sysvar::rent::ID)?;

    // Pay block opening fee.
    burn(
        sender_info,
        mint_quote_info,
        signer_info,
        token_program,
        OPEN_FEE,
    )?;

    // Error out if start slot is within the current period.
    let start_slot = id * 1500;
    let current_period_start = (clock.slot / 1500) * 1500;
    let current_period_end = current_period_start + 1500;
    if start_slot < current_period_end {
        return Err(ProgramError::InvalidArgument);
    }

    // Initialize config.
    create_program_account::<Block>(
        block_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[BLOCK, &id.to_le_bytes()],
    )?;
    let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    block.id = id;
    block.reward = RewardConfig {
        lode_hash: [0; 32],
        lode_authority: Pubkey::default(),
        lode_reward: 0,
        motherlode_threshold: MOTHERLOAD_DIFFICULTY,
        nugget_reward: 0,
        nugget_threshold: NUGGET_DIFFICULTY,
    };
    block.slot_hash = [0; 32];
    block.start_slot = start_slot;
    block.total_hashes = 0;

    // Select reward strategy.
    let noise_seed = block.id.to_le_bytes();
    let noise = hash(&noise_seed);
    let lode_reward = ONE_ORE * generate_lode(noise) as u64;
    let target_block_reward = ONE_ORE * 10;
    let expected_hashes_per_block = HASH_TOKEN_SUPPLY / 2;
    let expected_qualifying_hashes = expected_hashes_per_block / 2u64.pow(NUGGET_DIFFICULTY as u32);
    let difficulty_reward = (target_block_reward - lode_reward) / expected_qualifying_hashes;
    block.reward.lode_reward = lode_reward;
    block.reward.nugget_reward = difficulty_reward;

    // Initialize market.
    create_program_account::<Market>(
        market_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[MARKET, &id.to_le_bytes()],
    )?;
    let market = market_info.as_account_mut::<Market>(&ore_api::ID)?;
    market.base = TokenParams {
        mint: *mint_base_info.key,
        balance: HASH_TOKEN_SUPPLY,
        balance_virtual: 0,
    };
    market.quote = TokenParams {
        mint: *mint_quote_info.key,
        balance: 0,
        balance_virtual: VIRTUAL_LIQUIDITY,
    };
    market.fee = FeeParams {
        rate: FEE_RATE_BPS,
        uncollected: 0,
        cumulative: 0,
    };
    market.snapshot = Snapshot {
        enabled: 1,
        base_balance: 0,
        quote_balance: 0,
        slot: 0,
    };
    market.id = id;

    // Initialize hash token mint.
    let mint_bump = mint_pda(block.id).1;
    allocate_account_with_bump(
        mint_base_info,
        system_program,
        signer_info,
        spl_token::state::Mint::LEN,
        &spl_token::ID,
        &[MINT, &id.to_le_bytes()],
        mint_bump,
    )?;
    initialize_mint_signed_with_bump(
        mint_base_info,
        block_info,
        None,
        token_program,
        rent_sysvar,
        0,
        &[MINT, &id.to_le_bytes()],
        mint_bump,
    )?;

    // TODO Initialize hash token metadata.

    // Initialize collateral and commitment token accounts.
    if collateral_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            block_info,
            collateral_info,
            mint_quote_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        collateral_info.as_associated_token_account(block_info.key, mint_quote_info.key)?;
    }
    if commitment_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            block_info,
            commitment_info,
            mint_base_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        commitment_info.as_associated_token_account(block_info.key, mint_base_info.key)?;
    }

    // Initialize vault token accounts.
    if vault_base_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            market_info,
            vault_base_info,
            mint_base_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        vault_base_info.as_associated_token_account(market_info.key, mint_base_info.key)?;
    }
    if vault_quote_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            market_info,
            vault_quote_info,
            mint_quote_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        vault_quote_info.as_associated_token_account(market_info.key, mint_quote_info.key)?;
    }

    // Mint hash tokens to market.
    mint_to_signed(
        mint_base_info,
        vault_base_info,
        block_info,
        token_program,
        HASH_TOKEN_SUPPLY,
        &[BLOCK, &id.to_le_bytes()],
    )?;

    // Burn mint authority.
    set_authority_signed(
        mint_base_info,
        block_info,
        None,
        AuthorityType::MintTokens,
        token_program,
        &[BLOCK, &id.to_le_bytes()],
    )?;

    // Emit event.
    OpenEvent {
        disc: OreEvent::Open as u64,
        id,
        start_slot,
        signer: *signer_info.key,
        reward_config: block.reward,
        liquidity_base: market.base.liquidity() as u64,
        liquidity_quote: market.quote.liquidity() as u64,
        ts: clock.unix_timestamp,
    }
    .log_return();

    Ok(())
}

fn generate_lode(hash: [u8; 32]) -> u8 {
    // Extract the first byte (0 to 255)
    let byte_value = hash[0];

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
        let noise_seed = i.to_le_bytes();
        let noise = hash(&noise_seed);
        let lode_reward = ONE_ORE * generate_lode(noise) as u64;
        let target_block_reward = ONE_ORE * 10;
        let expected_hashes_per_block = HASH_TOKEN_SUPPLY / 2;
        let expected_qualifying_hashes =
            expected_hashes_per_block / 2u64.pow(NUGGET_DIFFICULTY as u32);
        let difficulty_reward = (target_block_reward - lode_reward) / expected_qualifying_hashes;
        println!("{}: {} {}", i, lode_reward, difficulty_reward);
    }
    // assert!(false);
}
