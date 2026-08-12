use entropy_api::state::Var;
use ore_api::prelude::*;
use solana_program::{keccak::hashv, log::sol_log, native_token::lamports_to_sol};
use steel::*;

/// Deploys capital to prospect on a square.
pub fn process_deploy(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Deploy::try_from_bytes(data)?;
    let mut amount = u64::from_le_bytes(args.amount);
    let mask = u32::from_le_bytes(args.squares);
    // Load accounts.
    let clock = Clock::get()?;
    let (ore_accounts, entropy_accounts) = accounts.split_at(10);
    sol_log(&format!("Ore accounts: {:?}", ore_accounts.len()).to_string());
    sol_log(&format!("Entropy accounts: {:?}", entropy_accounts.len()).to_string());
    let [signer_info, authority_info, automation_info, board_info, _config_info, miner_info, round_info, treasury_info, system_program, ore_program] =
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
        .has_address(&BOARD_ADDRESS)?
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.start_slot && clock.slot < b.end_slot)?;
    let round = round_info
        .has_seeds(&[ROUND, &board.round_id.to_le_bytes()], &ore_api::ID)?
        .as_account_mut::<Round>(&ore_api::ID)?
        .assert_mut(|r| r.id == board.round_id)?;
    let treasury = treasury_info
        .has_address(&TREASURY_ADDRESS)?
        .as_account_mut::<Treasury>(&ore_api::ID)?;
    miner_info
        .is_writable()?
        .has_seeds(&[MINER, &authority_info.key.to_bytes()], &ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Wait until first deploy to start round.
    if board.end_slot == u64::MAX {
        board.start_slot = clock.slot;
        board.end_slot = board.start_slot + ROUND_SLOTS;
        round.expires_at = board.end_slot + ONE_DAY_SLOTS;

        // Bump var to the next value.
        let [var_info, entropy_program] = entropy_accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };
        var_info
            .has_address(&VAR_ADDRESS)?
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

    // Get the automation.
    let mut strategy = u64::MAX;
    let automation = if !automation_info.data_is_empty() {
        let automation = automation_info
            .as_account_mut::<Automation>(&ore_api::ID)?
            .assert_mut(|a| a.executor == *signer_info.key || a.executor == EXECUTOR_ADDRESS)?
            .assert_mut(|a| a.authority == *authority_info.key)?;

        // Conditional deploy.
        let max_motherlode = automation.conditions.max_motherlode as u64 * ONE_ORE;
        let min_motherlode = automation.conditions.min_motherlode as u64 * ONE_ORE;
        if treasury.motherlode > max_motherlode || treasury.motherlode < min_motherlode {
            return Ok(());
        }

        // Set strategy.
        strategy = automation.strategy as u64;
        Some(automation)
    } else {
        None
    };

    // Update amount and mask for automation.
    let mut squares = [false; 25];
    if let Some(automation) = &automation {
        // Set amount and squares based on automation strategy.
        match AutomationStrategy::from_u64(automation.strategy as u64) {
            AutomationStrategy::Preferred => {
                // Preferred automation strategy. Use the miner authority's provided mask.
                amount = automation.amount;
                for i in 0..25 {
                    squares[i] = (automation.mask & (1 << i)) != 0;
                }
            }
            AutomationStrategy::Random => {
                // Random automation strategy. Generate a random mask based on number of squares user wants to deploy to.
                amount = automation.amount;

                // If first deploy, use the mask provided by the user.
                if automation.total_sol_spent == 0 {
                    for i in 0..25 {
                        squares[i] = (automation.mask & (1 << i)) != 0;
                    }
                } else if automation.conditions.solo_tiles > 0
                    || automation.conditions.split_tiles > 0
                {
                    // User has a preferred solo / split strategy. Generate a mask based on this preferrence.

                    // First generate the solo / split mask.
                    let distribution_mask = round.distribution_mask();

                    // Build squares array based on user's preferred solo and split tiles using fixed-size arrays.

                    // First collect the indices of solos and splits (maximum 25 of each).
                    let mut solo_idxs = [0usize; 25];
                    let mut split_idxs = [0usize; 25];
                    let mut num_solo = 0usize;
                    let mut num_split = 0usize;
                    for i in 0..25 {
                        if (distribution_mask & (1 << i)) != 0 {
                            solo_idxs[num_solo] = i;
                            num_solo += 1;
                        } else {
                            split_idxs[num_split] = i;
                            num_split += 1;
                        }
                    }

                    let solo_pref = automation.conditions.solo_tiles as usize;
                    let split_pref = automation.conditions.split_tiles as usize;

                    // Build a random seed using user and round id for deterministic shuffle
                    let mut seed = [0u8; 32];
                    let h = hashv(&[
                        &automation.authority.to_bytes(),
                        &round.id.to_le_bytes(),
                        b"solo_split_mask",
                    ])
                    .0;
                    seed.copy_from_slice(&h);

                    // Deterministic shuffle for solo and split index arrays in place
                    fn deterministic_shuffle(idxs: &mut [usize], len: usize, seed: &[u8; 32]) {
                        let mut local_seed = [0u8; 8];
                        for i in (1..len).rev() {
                            // Use 8 bytes from the 32-byte seed cycling per index
                            let start = (i % (32 - 8 + 1)) as usize;
                            local_seed.copy_from_slice(&seed[start..start + 8]);
                            let hashval = u64::from_le_bytes(local_seed);
                            let j = (hashval % (i as u64 + 1)) as usize;
                            // Swap i <-> j
                            let tmp = idxs[i];
                            idxs[i] = idxs[j];
                            idxs[j] = tmp;
                        }
                    }

                    // Apply shuffle to both solo and split slice parts
                    let (solo_idxs_slice, split_idxs_slice) =
                        (&mut solo_idxs[..num_solo], &mut split_idxs[..num_split]);
                    deterministic_shuffle(solo_idxs_slice, num_solo, &seed);
                    deterministic_shuffle(split_idxs_slice, num_split, &seed);

                    // Prepare the tiles to select
                    let num_solo_to_select = num_solo.min(solo_pref);
                    let num_split_to_select = num_split.min(split_pref);

                    // Clear current squares
                    for i in 0..25 {
                        squares[i] = false;
                    }

                    // Set true on chosen solo tiles
                    for n in 0..num_solo_to_select {
                        squares[solo_idxs[n]] = true;
                    }

                    // Set true on chosen split tiles
                    for n in 0..num_split_to_select {
                        squares[split_idxs[n]] = true;
                    }
                } else {
                    // If not first deploy and no preferred solo / split strategy, generate a random mask based on number of squares user wants to deploy to.
                    let num_squares = (0..25)
                        .filter(|i| (automation.mask & (1 << i)) != 0)
                        .count() as u64;
                    let r = hashv(&[&automation.authority.to_bytes(), &round.id.to_le_bytes()]).0;
                    squares = generate_random_mask(num_squares, &r);
                }
            }
            AutomationStrategy::Discretionary => {
                // Discretionary automation strategy. Use the executor's provided mask.
                amount = amount.min(automation.amount);
                for i in 0..25 {
                    squares[i] = (mask & (1 << i)) != 0;
                }
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
        miner.auto_return = 1;
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

    // Update total miners for round.
    let is_first_deploy = miner.deployed.iter().sum::<u64>() == 0;

    // Close automation if it doesn't have enough balance to cover all requested squares.
    if is_first_deploy {
        if let Some(automation) = &automation {
            let required_squares = squares.iter().filter(|&&s| s).count() as u64;
            if automation.balance < (amount * required_squares) + automation.fee {
                automation_info.send(automation.fee, &signer_info);
                automation_info.close(authority_info)?;
                return Ok(());
            }
        }
    }

    // Calculate all deployments.
    let mut total_amount = 0;
    let mut total_squares = 0;
    let mut deployed_squares = [false; 25];
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
        // round.total_deployed += amount;
        round.count[square_id] += 1;

        // Update totals.
        total_amount += amount;
        total_squares += 1;
        deployed_squares[square_id] = true;
    }

    // Update total miners for round.
    if is_first_deploy && total_amount > 0 {
        round.total_miners += 1;
    }

    // Increment miner lifetime deployed.
    miner.lifetime_deployed += total_amount;

    // Top up checkpoint fee.
    if miner.checkpoint_fee == 0 {
        miner.checkpoint_fee = CHECKPOINT_FEE;
        miner_info.collect(CHECKPOINT_FEE, &signer_info)?;
    }

    // Transfer SOL.
    if let Some(automation) = automation {
        // Update automation total sol spent.
        automation.total_sol_spent += total_amount;

        // Calculate automation fee.
        let automation_fee = if is_first_deploy && total_amount > 0 {
            automation.fee
        } else {
            0
        };

        // Update automation balance.
        automation.balance -= total_amount + automation_fee;
        automation_info.send(total_amount, &round_info);
        automation_info.send(automation_fee, &signer_info);

        // Close automation if balance is less than what's required to deploy 1 square.
        if automation.balance < automation.amount + automation.fee {
            automation_info.close(authority_info)?;
        }
    } else {
        round_info.collect(total_amount, &signer_info)?;
    }

    // Rebuild the mask from the deployed squares.
    let mut deployed_mask = 0;
    for (square_id, &deployed) in deployed_squares.iter().enumerate() {
        if deployed {
            deployed_mask |= 1 << square_id;
        }
    }

    // Log the deploy event.
    program_log(
        &[board_info.clone(), ore_program.clone()],
        DeployEvent {
            disc: 2,
            authority: miner.authority,
            amount,
            mask: deployed_mask as u64,
            round_id: round.id,
            signer: *signer_info.key,
            strategy,
            total_squares,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

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
