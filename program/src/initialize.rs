use ore_api::prelude::*;
use solana_program::program_pack::Pack;
use steel::*;

/// Sets the admin.
pub fn process_initialize(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Parse data.
    // let args = Initialize::try_from_bytes(data)?;

    // Load accounts.
    let [signer_info, config_info, market_info, mint_info, treasury_info, vault_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    config_info
        .is_writable()?
        .is_empty()?
        .has_seeds(&[CONFIG], &ore_api::ID)?;
    market_info
        .is_writable()?
        .is_empty()?
        .has_seeds(&[MARKET], &ore_api::ID)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    treasury_info
        .is_writable()?
        .is_empty()?
        .has_seeds(&[TREASURY], &ore_api::ID)?;
    vault_info
        .is_writable()?
        .is_empty()?
        .has_address(&vault_pda().0)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Create config account.
    create_program_account::<Config>(
        config_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[CONFIG],
    )?;
    let config = config_info.as_account_mut::<Config>(&ore_api::ID)?;
    config.admin = *signer_info.key;
    config.fee_collector = *signer_info.key;
    config.fee_rate = 0;

    // Initialize market.
    let initial_id: u64 = 0;
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
    market.block_id = initial_id;

    // Create treasury account.
    create_program_account::<Treasury>(
        treasury_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[TREASURY],
    )?;

    // Initialize vault token account.
    if vault_info.data_is_empty() {
        let vault_pda = vault_pda();
        allocate_account_with_bump(
            vault_info,
            system_program,
            signer_info,
            spl_token::state::Account::LEN,
            &spl_token::ID,
            &[
                market_info.key.as_ref(),
                token_program.key.as_ref(),
                mint_info.key.as_ref(),
            ],
            vault_pda.1,
        )?;
        solana_program::program::invoke(
            &spl_token_2022::instruction::initialize_account3(
                &spl_token::ID,
                &vault_pda.0,
                &mint_info.key,
                &market_info.key,
            )?,
            &[
                vault_info.clone(),
                mint_info.clone(),
                market_info.clone(),
                token_program.clone(),
            ],
        )?;
    } else {
        vault_info
            .has_address(&vault_pda().0)?
            .as_token_account()?
            .assert(|t| t.mint() == *mint_info.key)?
            .assert(|t| t.owner() == *market_info.key)?;
    }

    Ok(())
}
