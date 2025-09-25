use ore_api::prelude::*;
use solana_program::{keccak::hashv, log::sol_log, native_token::lamports_to_sol};
use steel::*;

use crate::whitelist::AUTHORIZED_ACCOUNTS;

/// Deploys capital to prospect on a square.
pub fn process_deploy(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Deploy::try_from_bytes(data)?;
    let mut amount = u64::from_le_bytes(args.amount);
    let mask = u32::from_le_bytes(args.squares);

    // Load accounts.
    let clock = Clock::get()?;
    let (required_accounts, miner_accounts) = accounts.split_at(9);
    let [signer_info, authority_info, automation_info, board_info, config_info, fee_collector_info, miner_info, square_info, system_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    authority_info.is_writable()?;
    automation_info.is_writable()?;
    let board = board_info
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| {
            (clock.slot >= b.start_slot && clock.slot < b.end_slot && b.slot_hash == [0; 32])
                || (clock.slot >= b.end_slot + INTERMISSION_SLOTS && b.slot_hash != [0; 32])
        })?;
    let config = config_info.as_account::<Config>(&ore_api::ID)?;
    fee_collector_info
        .has_address(&config.fee_collector)?
        .is_writable()?;
    miner_info.is_writable()?;
    let square = square_info.as_account_mut::<Square>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Check whitelist
    if !AUTHORIZED_ACCOUNTS.contains(&signer_info.key) {
        return Err(trace("Not authorized", OreError::NotAuthorized.into()));
    }

    // Check if signer is the automation executor.
    let automation = if !automation_info.data_is_empty() {
        let automation = automation_info
            .as_account_mut::<Automation>(&ore_api::ID)?
            .assert_mut(|a| a.executor == *signer_info.key)?
            .assert_mut(|a| a.authority == *authority_info.key)?;
        Some(automation)
    } else {
        None
    };

    // Update amount and mask for automation.
    let mut squares = [false; 25];
    if let Some(automation) = &automation {
        // Set amount
        amount = automation.amount;

        // Set squares
        match AutomationStrategy::from_u64(automation.strategy as u64) {
            AutomationStrategy::Preferred => {
                // Preferred automation strategy. Use the miner authority's provided mask.
                for i in 0..25 {
                    squares[i] = (automation.mask & (1 << i)) != 0;
                }
            }
            AutomationStrategy::Random => {
                // Random automation strategy. Generate a random mask based on number of squares user wants to deploy to.
                let num_squares = ((automation.mask & 0xFF) as u64).min(25);
                let r = hashv(&[&automation.authority.to_bytes(), &board.id.to_le_bytes()]).0;
                squares = generate_random_mask(num_squares, &r);
            }
        }
    } else {
        // Convert provided 32-bit mask into array of 25 booleans, where each bit in the mask
        // determines if that square index is selected (true) or not (false)
        for i in 0..25 {
            squares[i] = (mask & (1 << i)) != 0;
        }
    }

    // Log
    sol_log(
        &format!(
            "Deploying {} SOL to {} squares",
            lamports_to_sol(amount),
            squares.iter().filter(|&&s| s).count(),
        )
        .as_str(),
    );

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
        miner.deployed = [0; 25];
        miner.rewards_sol = 0;
        miner.rewards_ore = 0;
        miner.round_id = board.id;
        miner.lifetime_rewards_sol = 0;
        miner.lifetime_rewards_ore = 0;
        miner
    } else {
        miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| {
                if let Some(automation) = &automation {
                    // only run automation once per round
                    m.authority == automation.authority
                        && (m.round_id < board.id || board.slot_hash != [0; 32])
                } else {
                    m.authority == *signer_info.key
                }
            })?
    };

    // Reset board.
    if board.slot_hash != [0; 32] {
        // Reset board
        board.deployed = [0; 25];
        board.id += 1;
        board.slot_hash = [0; 32];
        board.start_slot = clock.slot;
        board.end_slot = clock.slot + 150; // one minute
        board.top_miner = Pubkey::default();
        board.total_deployed = 0;
        board.total_vaulted = 0;
        board.total_winnings = 0;

        // Reset squares
        square.count = [0; 25];
        square.miners = [[Pubkey::default(); 16]; 25];
    }

    // Reset miner
    if miner.round_id != board.id {
        miner.deployed = [0; 25];
        miner.round_id = board.id;
    }

    // Normalize amount.
    let fee = amount / 100;
    let amount = amount - fee;

    // Calculate all deployments.
    let mut refund_amounts = [0; 25];
    let mut refund_miner_infos = [None; 25];
    let mut total_fee = 0;
    let mut total_amount = 0;
    'deploy: for (square_id, &should_deploy) in squares.iter().enumerate() {
        // Skip if square index is out of bounds.
        if square_id > 24 {
            break;
        }

        // Skip if square is not deployed to.
        if !should_deploy {
            continue;
        }

        // Get deployment metadata.
        let is_first_move = miner.deployed[square_id] == 0;
        let mut idx = if is_first_move {
            // Insert at end of the list.
            square.count[square_id] as usize
        } else {
            // Find the miner's index in the list.
            let mut idx = 0;
            for i in 0..16 {
                if square.miners[square_id][i] == miner.authority {
                    idx = i;
                    break;
                }
            }
            idx
        };

        // If the square is full, refund the miner with the smallest deployment and kick them off the square.
        if idx == 16 {
            // Find miner with the smallest deployment.
            let mut smallest_miner = Pubkey::default();
            let mut smallest_deployment = u64::MAX;
            for i in 0..16 {
                if square.deployed[square_id][i] < smallest_deployment {
                    smallest_deployment = square.deployed[square_id][i];
                    smallest_miner = square.miners[square_id][i];
                    idx = i;
                }
            }

            // Safety check.
            // This should never happen.
            assert!(smallest_miner != Pubkey::default());

            // If deploy amount is less than smallest deployment, skip this square.
            if amount < smallest_deployment {
                continue 'deploy;
            }

            // Refund the smallest miner and kick them off the square.
            for miner_info in miner_accounts {
                if *miner_info.key == miner_pda(smallest_miner).0 {
                    let smallest_miner = miner_info
                        .as_account_mut::<Miner>(&ore_api::ID)?
                        .assert_mut(|m| m.authority == smallest_miner)?;
                    smallest_miner.refund_sol += smallest_deployment;
                    smallest_miner.deployed[square_id] -= smallest_deployment;
                    refund_amounts[square_id] = smallest_deployment;
                    refund_miner_infos[square_id] = Some(miner_info);

                    // Remove smallest miner from square
                    board.deployed[square_id] -= smallest_deployment;
                    board.total_deployed -= smallest_deployment;
                    square.deployed[square_id][idx] -= smallest_deployment;
                    square.miners[square_id][idx] = Pubkey::default();
                    square.count[square_id] -= 1;

                    break;
                }
            }
        }

        // Safety check.
        // This should never happen.
        assert!(idx < 16);

        // Safety check.
        // Skip if square count is >= 16. This can only happen if the signer didn't provide a miner account to refund.
        if square.count[square_id] >= 16 {
            continue 'deploy;
        }

        // Update miner
        miner.deployed[square_id] += amount;

        // Update square
        if is_first_move {
            square.miners[square_id][idx] = miner.authority;
            square.count[square_id] += 1;
        }

        // Update board
        board.deployed[square_id] += amount;
        board.total_deployed += amount;

        // Update square deployed
        square.deployed[square_id][idx] += amount;

        // Update total fee and amount
        total_fee += fee;
        total_amount += amount;
    }

    // Transfer SOL refunds.
    for (square_id, refund_amount) in refund_amounts.iter().enumerate() {
        if let Some(refund_miner_info) = refund_miner_infos[square_id] {
            board_info.send(*refund_amount, &refund_miner_info);
        }
    }

    // Transfer SOL.
    if let Some(automation) = automation {
        automation.balance -= total_amount + total_fee + automation.fee;
        automation_info.send(total_amount, &board_info);
        automation_info.send(total_fee, &fee_collector_info);
        automation_info.send(automation.fee, &signer_info);

        // Close automation if balance is 0.
        if automation.balance == 0 {
            automation_info.close(authority_info)?;
        }
    } else {
        board_info.collect(total_amount, &signer_info)?;
        fee_collector_info.collect(total_fee, &signer_info)?;
    }

    Ok(())
}

fn generate_random_mask(num_squares: u64, r: &[u8]) -> [bool; 25] {
    let mut new_mask = [false; 25];
    let mut selected = 0;
    for i in 0..25 {
        let rand_byte = r[i];
        let remaining_needed = num_squares as u64 - selected as u64;
        let remaining_positions = 25 - i;
        if remaining_needed > 0
            && (rand_byte as u64) * (remaining_positions as u64) < (remaining_needed * 256)
        {
            new_mask[i] = true;
            selected += 1;
        }
    }
    new_mask
}
