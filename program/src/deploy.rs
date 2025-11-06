use entropy_api::state::Var;
use ore_api::prelude::*;
use solana_program::{keccak::hashv, log::sol_log, native_token::lamports_to_sol};
use steel::*;

use crate::reset::ORE_VAR_ADDRESS;

/// Deploys capital to prospect on a square.
pub fn process_deploy(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Deploy::try_from_bytes(data)?;
    let mut amount = u64::from_le_bytes(args.amount);
    let mask = u32::from_le_bytes(args.squares);

    // Load accounts.
    let clock = Clock::get()?;
    let (ore_accounts, entropy_accounts) = accounts.split_at(7);
    sol_log(&format!("Ore accounts: {:?}", ore_accounts.len()).to_string());
    sol_log(&format!("Entropy accounts: {:?}", entropy_accounts.len()).to_string());
    let [signer_info, authority_info, automation_info, board_info, miner_info, round_info, system_program] =
        ore_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    authority_info.is_writable()?;
    automation_info
        .is_writable()?
        .has_seeds(&[AUTOMATION, &authority_info.key.to_bytes()], &ore_api::ID)?;
    let board = board_info
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot && clock.slot < b.end_slot)?;
    let round = round_info
        .as_account_mut::<Round>(&ore_api::ID)?
        .assert_mut(|r| r.id == board.round_id)?;
    miner_info
        .is_writable()?
        .has_seeds(&[MINER, &authority_info.key.to_bytes()], &ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Wait until first deploy to start round.
    if board.end_slot == u64::MAX {
        board.start_slot = clock.slot;
        board.end_slot = board.start_slot + 150;
        round.expires_at = board.end_slot + ONE_DAY_SLOTS;

        // Bump var to the next value.
        let [var_info, entropy_program] = entropy_accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        var_info
            .has_address(&ORE_VAR_ADDRESS)?
            .as_account::<Var>(&entropy_api::ID)?
            .assert(|v| v.authority == *board_info.key)?;
        entropy_program.is_program(&entropy_api::ID)?;

        // Bump var to the next value.
        invoke_signed(
            &entropy_api::sdk::next(*board_info.key, *var_info.key, board.end_slot),
            &[board_info.clone(), var_info.clone()],
            &entropy_api::ID,
            &[BOARD],
        )?;
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
                let r = hashv(&[&automation.authority.to_bytes(), &round.id.to_le_bytes()]).0;
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

    // Open miner account.
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
        miner.cumulative = [0; 25];
        miner.rewards_sol = 0;
        miner.rewards_ore = 0;
        miner.round_id = 0;
        miner.checkpoint_id = 0;
        miner.lifetime_rewards_sol = 0;
        miner.lifetime_rewards_ore = 0;
        miner
    } else {
        miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| {
                if let Some(automation) = &automation {
                    m.authority == automation.authority
                } else {
                    m.authority == *signer_info.key
                }
            })?
    };

    // Reset miner
    if miner.round_id != round.id {
        // Assert miner has checkpointed prior round.
        assert!(
            miner.checkpoint_id == miner.round_id,
            "Miner has not checkpointed"
        );

        // Reset miner for new round.
        miner.deployed = [0; 25];
        miner.cumulative = round.deployed;
        miner.round_id = round.id;
    }

    // Calculate all deployments.
    let mut total_amount = 0;
    let mut total_squares = 0;
    for (square_id, &should_deploy) in squares.iter().enumerate() {
        // Skip if square index is out of bounds.
        if square_id > 24 {
            break;
        }

        // Skip if square is not deployed to.
        if !should_deploy {
            continue;
        }

        // Skip if miner already deployed to this square.
        if miner.deployed[square_id] > 0 {
            continue;
        }

        // Record cumulative amount.
        miner.cumulative[square_id] = round.deployed[square_id];

        // Update miner
        miner.deployed[square_id] = amount;

        // Update board
        round.deployed[square_id] += amount;
        round.total_deployed += amount;
        round.count[square_id] += 1;

        // Update totals.
        total_amount += amount;
        total_squares += 1;

        // Exit early if automation does not have enough balance for another square.
        if let Some(automation) = &automation {
            if total_amount + automation.fee + amount > automation.balance {
                break;
            }
        }
    }

    // Top up checkpoint fee.
    if miner.checkpoint_fee == 0 {
        miner.checkpoint_fee = CHECKPOINT_FEE;
        miner_info.collect(CHECKPOINT_FEE, &signer_info)?;
    }

    // Transfer SOL.
    if let Some(automation) = automation {
        automation.balance -= total_amount + automation.fee;
        automation_info.send(total_amount, &round_info);
        automation_info.send(automation.fee, &signer_info);

        // Close automation if balance is less than what's required to deploy 1 square.
        if automation.balance < automation.amount + automation.fee {
            automation_info.close(authority_info)?;
        }
    } else {
        round_info.collect(total_amount, &signer_info)?;
    }

    // Log
    sol_log(
        &format!(
            "Round #{}: deploying {} SOL to {} squares",
            round.id,
            lamports_to_sol(amount),
            total_squares,
        )
        .as_str(),
    );

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
