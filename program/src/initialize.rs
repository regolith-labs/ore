use ore_api::prelude::*;
use steel::*;

/// Sets the admin.
pub fn process_initialize(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Parse data.
    // let args = Initialize::try_from_bytes(data)?;

    // Load accounts.
    let [signer_info, config_info, market_info, treasury_info, system_program] = accounts else {
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
    treasury_info
        .is_writable()?
        .is_empty()?
        .has_seeds(&[TREASURY], &ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

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

    Ok(())
}
