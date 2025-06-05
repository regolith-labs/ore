use ore_api::prelude::*;
use solana_program::program_pack::Pack;
use steel::*;

/// Opens a new block.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Open::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);

    // Load accounts.
    let [signer_info, block_info, market_info, market_hash_info, market_ore_info, mint_hash_info, mint_ore_info, system_program, token_program, associated_token_program, rent_sysvar] =
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
    mint_hash_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[MINT, &id.to_le_bytes()], &ore_api::ID)?;
    mint_ore_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;
    rent_sysvar.is_sysvar(&sysvar::rent::ID)?;

    // Initialize config.
    create_program_account::<Block>(
        block_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[BLOCK, &id.to_le_bytes()],
    )?;
    let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    block.best_hash = [0; 32];
    block.best_miner = Pubkey::default();
    block.id = id;
    block.reward = ONE_ORE * 10;
    block.slot_hash = [0; 32];
    block.start_slot = 1500 * id;

    // Initialize market.
    create_program_account::<Market>(
        market_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[MARKET, &id.to_le_bytes()],
    )?;
    let market = market_info.as_account_mut::<Market>(&ore_api::ID)?;
    market.id = id;

    // Initialize hash token mint.
    allocate_account(
        mint_hash_info,
        system_program,
        signer_info,
        spl_token::state::Mint::LEN,
        &spl_token::ID,
        &[MINT, &id.to_le_bytes()],
    )?;
    initialize_mint_signed(
        mint_hash_info,
        block_info,
        None,
        token_program,
        rent_sysvar,
        0,
        &[MINT, &id.to_le_bytes()],
    )?;

    // TODO Initialize hash token metadata.

    // Initialize token accounts for market.
    if market_hash_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            market_info,
            market_hash_info,
            mint_hash_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        market_hash_info.as_associated_token_account(market_info.key, mint_hash_info.key)?;
    }
    if market_ore_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            market_info,
            market_ore_info,
            mint_ore_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        market_ore_info.as_associated_token_account(market_info.key, mint_ore_info.key)?;
    }

    // Mint hash tokens to market.
    mint_to_signed(
        mint_hash_info,
        market_hash_info,
        block_info,
        token_program,
        10_000_000,
        &[BLOCK, &id.to_le_bytes()],
    )?;

    // Burn mint authority.
    set_authority_signed(
        mint_hash_info,
        block_info,
        None,
        spl_token_2022::instruction::AuthorityType::MintTokens,
        token_program,
        &[BLOCK, &id.to_le_bytes()],
    )?;

    Ok(())
}
