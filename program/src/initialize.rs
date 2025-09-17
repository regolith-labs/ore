use ore_api::prelude::*;
use steel::*;

/// Initializes the program.
pub fn process_initialize(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, board_info, config_info, mint_info, treasury_info, vault_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    board_info.has_seeds(&[BOARD], &ore_api::ID)?;
    config_info.has_seeds(&[CONFIG], &ore_api::ID)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    treasury_info.has_seeds(&[TREASURY], &ore_api::ID)?;
    vault_info.has_address(&vault_address())?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Create board account.
    if board_info.data_is_empty() {
        create_program_account::<Board>(
            board_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[BOARD],
        )?;
        let board = board_info.as_account_mut::<Board>(&ore_api::ID)?;
        board.commits = [0; 25];
        board.id = 0;
        board.start_at = 0;
        board.start_slot = 0;
        board.end_slot = 0;
        board.slot_hash = [0; 32];
        board.total_commits = 0;
        board.total_burned = 0;
    } else {
        board_info.as_account::<Board>(&ore_api::ID)?;
    }

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

    // Initialize vault token account.
    if vault_info.data_is_empty() {
        create_associated_token_account(
            signer_info,
            board_info,
            vault_info,
            mint_info,
            system_program,
            token_program,
            associated_token_program,
        )?;
    } else {
        vault_info.as_associated_token_account(board_info.key, mint_info.key)?;
    }

    Ok(())
}
