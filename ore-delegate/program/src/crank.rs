use ore_delegate_api::prelude::*;
use solana_program::program_pack::Pack;
use spl_token_2022::instruction::AuthorityType;
use steel::*;

/// Cranks a mining transaction.
pub fn process_crank(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Crank::try_from_bytes(data)?;
    // let id = u64::from_le_bytes(args.id);

    // Load accounts.
    // let [signer_info, block_info, market_info, mint_base_info, mint_quote_info, system_program, vault_base_info, vault_quote_info, token_program, associated_token_program, rent_sysvar] =
    //     accounts
    // else {
    //     return Err(ProgramError::NotEnoughAccountKeys);
    // };
    // signer_info.is_signer()?;
    // block_info
    //     .is_empty()?
    //     .is_writable()?
    //     .has_seeds(&[BLOCK, &id.to_le_bytes()], &ore_api::ID)?;
    // market_info
    //     .is_empty()?
    //     .is_writable()?
    //     .has_seeds(&[MARKET, &id.to_le_bytes()], &ore_api::ID)?;
    // mint_base_info
    //     .is_empty()?
    //     .is_writable()?
    //     .has_seeds(&[MINT, &id.to_le_bytes()], &ore_api::ID)?;
    // mint_quote_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    // system_program.is_program(&system_program::ID)?;
    // token_program.is_program(&spl_token::ID)?;
    // associated_token_program.is_program(&spl_associated_token_account::ID)?;
    // rent_sysvar.is_sysvar(&sysvar::rent::ID)?;

    // // Initialize config.
    // create_program_account::<Block>(
    //     block_info,
    //     system_program,
    //     signer_info,
    //     &ore_api::ID,
    //     &[BLOCK, &id.to_le_bytes()],
    // )?;
    // let block = block_info.as_account_mut::<Block>(&ore_api::ID)?;
    // block.best_hash = [0; 32];
    // block.best_miner = Pubkey::default();
    // block.id = id;
    // block.reward = ONE_ORE * 10;
    // block.slot_hash = [0; 32];
    // block.start_slot = 1500 * id;

    // // Initialize market.
    // create_program_account::<Market>(
    //     market_info,
    //     system_program,
    //     signer_info,
    //     &ore_api::ID,
    //     &[MARKET, &id.to_le_bytes()],
    // )?;
    // let market = market_info.as_account_mut::<Market>(&ore_api::ID)?;
    // market.base = TokenParams {
    //     mint: *mint_base_info.key,
    //     balance: HASH_TOKEN_SUPPLY,
    //     balance_virtual: 0,
    // };
    // market.quote = TokenParams {
    //     mint: *mint_quote_info.key,
    //     balance: 0,
    //     balance_virtual: ONE_ORE,
    // };
    // market.fee = FeeParams {
    //     rate: FEE_RATE_BPS,
    //     uncollected: 0,
    //     cumulative: 0,
    // };
    // market.snapshot = Snapshot {
    //     enabled: 1,
    //     base_balance: 0,
    //     quote_balance: 0,
    //     slot: 0,
    // };
    // market.id = id;

    // // Initialize hash token mint.
    // allocate_account(
    //     mint_base_info,
    //     system_program,
    //     signer_info,
    //     spl_token::state::Mint::LEN,
    //     &spl_token::ID,
    //     &[MINT, &id.to_le_bytes()],
    // )?;
    // initialize_mint_signed(
    //     mint_base_info,
    //     block_info,
    //     None,
    //     token_program,
    //     rent_sysvar,
    //     0,
    //     &[MINT, &id.to_le_bytes()],
    // )?;

    // // TODO Initialize hash token metadata.

    // // Initialize vault token accounts.
    // if vault_base_info.data_is_empty() {
    //     create_associated_token_account(
    //         signer_info,
    //         market_info,
    //         vault_base_info,
    //         mint_base_info,
    //         system_program,
    //         token_program,
    //         associated_token_program,
    //     )?;
    // } else {
    //     vault_base_info.as_associated_token_account(market_info.key, mint_base_info.key)?;
    // }
    // if vault_quote_info.data_is_empty() {
    //     create_associated_token_account(
    //         signer_info,
    //         market_info,
    //         vault_quote_info,
    //         mint_quote_info,
    //         system_program,
    //         token_program,
    //         associated_token_program,
    //     )?;
    // } else {
    //     vault_quote_info.as_associated_token_account(market_info.key, mint_quote_info.key)?;
    // }

    // // Mint hash tokens to market.
    // mint_to_signed(
    //     mint_base_info,
    //     vault_base_info,
    //     block_info,
    //     token_program,
    //     HASH_TOKEN_SUPPLY,
    //     &[BLOCK, &id.to_le_bytes()],
    // )?;

    // // Burn mint authority.
    // set_authority_signed(
    //     mint_base_info,
    //     block_info,
    //     None,
    //     AuthorityType::MintTokens,
    //     token_program,
    //     &[BLOCK, &id.to_le_bytes()],
    // )?;

    Ok(())
}
