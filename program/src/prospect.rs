use ore_api::prelude::*;
use steel::*;

/// Claims a block reward.
pub fn process_prospect(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Prospect::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);
    let square_id = u64::from_le_bytes(args.square_id);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, board_info, config_info, fee_collector_info, miner_info, sender_info, square_info, vault_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| {
            (clock.slot >= b.start_slot && clock.slot < b.end_slot && b.slot_hash == [0; 32])
                || (clock.slot >= b.end_slot + 25 && b.slot_hash != [0; 32])
        })?;
    let config = config_info.as_account::<Config>(&ore_api::ID)?;
    fee_collector_info
        .has_address(&config.fee_collector)?
        .is_writable()?;
    miner_info.is_writable()?;
    sender_info.as_associated_token_account(&signer_info.key, &MINT_ADDRESS)?;
    let square = square_info
        .as_account_mut::<Square>(&ore_api::ID)?
        .assert_mut(|s| s.id == square_id)?;
    vault_info.has_address(&vault_address())?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Create miner.
    let miner = if miner_info.data_is_empty() {
        create_program_account::<Miner>(
            miner_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[MINER, &signer_info.key.to_bytes()],
        )?;
        let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
        miner.authority = *signer_info.key;
        miner.commits = [0; 25];
        miner.rewards = 0;
        miner.round_id = board.id;
        miner.total_rewards = 0;
        miner
    } else {
        miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.authority == *signer_info.key)?
    };

    // Reset board.
    if board.slot_hash != [0; 32] {
        board.commits = [0; 25];
        board.id += 1;
        board.slot_hash = [0; 32];
        board.start_slot = clock.slot;
        board.end_slot = clock.slot + 150; // one minute
        board.total_commits = 0;
        board.total_burned = 0;
    }

    // Reset miner
    if miner.round_id != board.id {
        miner.commits = [0; 25];
        miner.round_id = board.id;
    }

    // Reset square
    if square.round_id != board.id {
        square.count = 0;
        square.miners = [Pubkey::default(); 16];
        square.round_id = board.id;
    }

    // Update miner
    let is_first_play_on_square = miner.commits[square_id as usize] == 0;
    miner.commits[square_id as usize] += amount;

    // Update square
    if is_first_play_on_square {
        square.miners[square.count as usize] = *signer_info.key;
        square.count += 1;
    }

    // Update board
    board.commits[square_id as usize] += amount;
    board.total_commits += amount;

    // Transfer tokens.
    transfer(signer_info, sender_info, vault_info, token_program, amount)?;

    // Pay fee.
    if config.fee_rate > 0 {
        fee_collector_info.collect(config.fee_rate, &signer_info)?;
    }

    Ok(())
}
