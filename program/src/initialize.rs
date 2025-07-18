use ore_api::prelude::*;
use steel::*;

/// Initializes the program.
pub fn process_initialize(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, market_info, mint_info, treasury_info, treasury_tokens_info, vault_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    config_info.has_seeds(&[CONFIG], &ore_api::ID)?;
    market_info.has_seeds(&[MARKET], &ore_api::ID)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    treasury_info.has_seeds(&[TREASURY], &ore_api::ID)?;
    treasury_tokens_info.is_writable()?;
    vault_info.has_address(&vault_address())?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Create config account.
    if config_info.data_is_empty() {
        create_program_account::<Config>(
            config_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[CONFIG],
        )?;
        let config = config_info.as_account_mut::<Config>(&ore_api::ID)?;
        config.admin = *signer_info.key;
        config.block_duration = INITIAL_BLOCK_DURATION;
        config.sniper_fee_duration = INITIAL_SNIPER_FEE_DURATION;
        config.fee_collector = *signer_info.key;
        config.fee_rate = FEE_LAMPORTS;
    } else {
        config_info.as_account::<Config>(&ore_api::ID)?;
    }

    // Initialize market.
    if market_info.data_is_empty() {
        create_program_account::<Market>(
            market_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[MARKET],
        )?;
        let market = market_info.as_account_mut::<Market>(&ore_api::ID)?;
        market.base = TokenParams {
            mint: Pubkey::default(), // Virtual token
            balance: 0,
            balance_virtual: 0,
        };
        market.quote = TokenParams {
            mint: MINT_ADDRESS,
            balance: 0,
            balance_virtual: 0,
        };
        market.fee = FeeParams {
            rate: 0,
            uncollected: 0,
            cumulative: 0,
        };
        market.snapshot = Snapshot {
            enabled: 1,
            base_balance: 0,
            quote_balance: 0,
            slot: 0,
        };
        market.block_id = 0;
    } else {
        market_info.as_account::<Market>(&ore_api::ID)?;
    }

    // Create treasury account.
    if treasury_info.data_is_empty() {
        create_program_account::<Treasury>(
            treasury_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[TREASURY],
        )?;
    } else {
        treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    }

    // Load treasury tokens.
    if treasury_tokens_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            treasury_info,
            treasury_tokens_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        treasury_tokens_info.as_associated_token_account(treasury_info.key, mint_info.key)?;
    }

    // Initialize vault token account.
    if vault_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            market_info,
            vault_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        vault_info.as_associated_token_account(market_info.key, mint_info.key)?;
    }

    Ok(())
}
