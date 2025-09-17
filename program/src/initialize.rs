use ore_api::prelude::*;
use steel::*;

/// Initializes the program.
pub fn process_initialize(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, board_info, config_info, mint_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    board_info.has_seeds(&[BOARD], &ore_api::ID)?;
    config_info.has_seeds(&[CONFIG], &ore_api::ID)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    treasury_info.has_seeds(&[TREASURY], &ore_api::ID)?;
    treasury_tokens_info.has_address(&treasury_tokens_address())?;
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
        board.prospects = [0; 25];
        board.id = 0;
        board.start_at = 0;
        board.start_slot = 0;
        board.end_slot = 0;
        board.slot_hash = [0; 32];
        board.top_miner = Pubkey::default();
        board.total_prospects = 0;
        board.total_vaulted = 0;
        board.total_winnings = 0;
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
        config.block_duration = 0;
        config.sniper_fee_duration = 0;
        config.fee_collector = *signer_info.key;
        config.fee_rate = 0;
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
        let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
        treasury.balance = 0;
    } else {
        treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    }

    // Initialize vault token account.
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

    Ok(())
}
