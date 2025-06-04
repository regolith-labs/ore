use ore_api::prelude::*;
use steel::*;

/// Opens a new block for hashpower trading.
pub fn process_open(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Open::try_from_bytes(data)?;
    let id = u64::from_le_bytes(args.id);

    // Load accounts.
    let [signer_info, block_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    block_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BLOCK, &id.to_le_bytes()], &ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

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

    // TODO Init market
    // TODO Init hash token mint
    // TODO Init token accounts for market
    // TODO Init mint hash tokens to market

    // // Initialize block token accounts.
    // create_associated_token_account(
    //     signer_info,
    //     block_info,
    //     block_commits_info,
    //     sol_mint_info,
    //     system_program,
    //     token_program,
    //     associated_token_program,
    // )?;
    // create_associated_token_account(
    //     signer_info,
    //     block_info,
    //     block_ore_info,
    //     ore_mint_info,
    //     system_program,
    //     token_program,
    //     associated_token_program,
    // )?;

    Ok(())
}
