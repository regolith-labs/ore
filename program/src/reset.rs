use ore_api::prelude::*;
use solana_program::{log::sol_log, slot_hashes::SlotHashes};
use steel::*;

/// Pays out the winners and block reward.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let (required_accounts, miner_accounts) = accounts.split_at(12);
    let [signer_info, board_info, config_info, fee_collector_info, mint_info, square_info, treasury_info, treasury_tokens_info, system_program, token_program, ore_program, slot_hashes_sysvar] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| b.slot_hash == [0; 32])?
        .assert_mut(|b| clock.slot > b.end_slot)?;
    let config = config_info.as_account::<Config>(&ore_api::ID)?;
    fee_collector_info
        .is_writable()?
        .has_address(&config.fee_collector)?;
    let mint = mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let square = square_info.as_account_mut::<Square>(&ore_api::ID)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info.as_associated_token_account(&treasury_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Sample slot hash.
    let mut r = 0;
    let (winning_square, square_deployed) =
        if let Ok(slot_hash) = get_slot_hash(board.end_slot, slot_hashes_sysvar) {
            board.slot_hash = slot_hash;
            r = rng(&slot_hash);
            let winning_square = get_winning_square(r);
            let square_deployed = board.deployed[winning_square];
            (winning_square, square_deployed)
        } else {
            // Cannot get slot hash. No one wins.
            board.slot_hash = [u8::MAX; 32];
            (u64::MAX as usize, 0)
        };

    // Collect fees.
    let fee = board.total_deployed / 100;

    // No one won. Vault all deployed.
    if square_deployed == 0 {
        // Update board.
        board.total_vaulted = board.total_deployed;
        treasury.balance += board.total_deployed - fee;

        // Emit event.
        program_log(
            &[board_info.clone(), ore_program.clone()],
            ResetEvent {
                disc: 0,
                round_id: board.id,
                start_slot: board.start_slot,
                end_slot: board.end_slot,
                winning_square: winning_square as u64,
                top_miner: board.top_miner,
                num_winners: 0,
                total_deployed: board.total_deployed,
                total_vaulted: board.total_vaulted,
                total_winnings: board.total_winnings,
                total_minted: 0,
                ts: clock.unix_timestamp,
            }
            .to_bytes(),
        )?;

        // Do SOL transfers.
        board_info.send(fee, &fee_collector_info);
        board_info.send(board.total_deployed - fee, &treasury_info);
        return Ok(());
    }

    // Get winnings amount (deployed on all non-winning squares).
    let mut winnings = 0;
    for (i, deployed) in board.deployed.iter().enumerate() {
        if i != winning_square {
            let fee = deployed / 100;
            winnings += deployed - fee;
        }
    }

    // Get vault amount (backing ORE floor price with SOL).
    let vault_amount = winnings / 10; // 10% of winnings.
    let winnings = winnings - vault_amount;
    board.total_vaulted = vault_amount;
    treasury.balance += vault_amount;

    // Assert
    assert!(board.total_deployed >= vault_amount + winnings + fee);

    // Check if motherlode was activated.
    let motherlode_activated = is_motherlode_activated(r);

    // Record miner rewards.
    let mut miner_deployments = [0; 16];
    let mut rewards_sol = [0; 16];
    let mut checksum = 0;
    for (i, miner_info) in miner_accounts.iter().enumerate() {
        // Load miner.
        let miner = miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.round_id == board.id)?
            .assert_mut(|m| m.authority == square.miners[winning_square][i])?;

        // Record SOL winnings.
        let miner_deployed = miner.deployed[winning_square];
        let fee = miner_deployed / 100;
        let miner_winnings =
            ((winnings as u128 * miner_deployed as u128) / square_deployed as u128) as u64;
        let rewards = (miner_deployed - fee) + miner_winnings; // Winners get their own deployment back, minus fee, plus their share of the winnings.
        checksum += miner_deployed;
        miner.rewards_sol += rewards;
        miner.lifetime_rewards_sol += rewards;
        rewards_sol[i] = rewards;
        miner_deployments[i] = miner_deployed;

        // Record ORE motherlode winnings.
        if motherlode_activated {
            let motherlode_winnings = ((treasury.motherlode as u128 * miner_deployed as u128)
                / square_deployed as u128) as u64;
            miner.rewards_ore += motherlode_winnings;
            miner.lifetime_rewards_ore += motherlode_winnings;
        }
    }

    // Safety check.
    // This can only happen if the caller didn't provide a complete set of winning miner accounts.
    assert!(checksum == square_deployed, "Invalid checksum");

    // Payout 1 ORE to the winning miner.
    // Each miner's chance of winning is proportional to their deposit amount relative to total deposits on the winning square.
    let mut mint_amount = 0;
    if let Some(i) = get_winning_miner(r, square_deployed, miner_deployments) {
        let miner = miner_accounts[i].as_account_mut::<Miner>(&ore_api::ID)?;
        mint_amount = ONE_ORE.min(MAX_SUPPLY - mint.supply());
        if mint_amount > 0 {
            // Record rewards of the winning miner.
            miner.rewards_ore += mint_amount;
            miner.lifetime_rewards_ore += mint_amount;
            board.top_miner = miner.authority;

            // Mint 1 ORE for the winning miner.
            mint_to_signed(
                mint_info,
                treasury_tokens_info,
                treasury_info,
                token_program,
                mint_amount,
                &[TREASURY],
            )?;
        }
    }

    // Reset the motherlode if it was activated.
    if motherlode_activated {
        // Log the motherlode event.
        program_log(
            &[board_info.clone(), ore_program.clone()],
            MotherlodeEvent {
                disc: 2,
                amount: treasury.motherlode,
                round_id: board.id,
                num_miners: square.count[winning_square],
                ts: clock.unix_timestamp,
            }
            .to_bytes(),
        )?;

        // Reset the motherlode.
        treasury.motherlode = 0;
    }

    // Top up the motherlode rewards pool.
    let mint = mint_info.as_mint()?;
    let motherlode_mint_amount = (ONE_ORE / 5).min(MAX_SUPPLY - mint.supply());
    if motherlode_mint_amount > 0 {
        mint_to_signed(
            mint_info,
            treasury_tokens_info,
            treasury_info,
            token_program,
            motherlode_mint_amount,
            &[TREASURY],
        )?;
        treasury.motherlode += motherlode_mint_amount;
    }

    // Update board.
    board.total_winnings = winnings;

    // Emit event.
    program_log(
        &[board_info.clone(), ore_program.clone()],
        ResetEvent {
            disc: 0,
            round_id: board.id,
            start_slot: board.start_slot,
            end_slot: board.end_slot,
            winning_square: winning_square as u64,
            top_miner: board.top_miner,
            num_winners: square.count[winning_square],
            total_deployed: board.total_deployed,
            total_vaulted: board.total_vaulted,
            total_winnings: board.total_winnings,
            total_minted: mint_amount + motherlode_mint_amount,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

    // Do SOL transfers.
    board_info.send(fee, &fee_collector_info);
    board_info.send(vault_amount, &treasury_info);
    for (i, miner_info) in miner_accounts.iter().enumerate() {
        board_info.send(rewards_sol[i], &miner_info);
    }

    Ok(())
}

pub fn get_slot_hash(
    slot: u64,
    slot_hashes_sysvar: &AccountInfo<'_>,
) -> Result<[u8; 32], ProgramError> {
    let slot_hashes =
        bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
    let Some(slot_hash) = slot_hashes.get(&slot) else {
        // If reset is not called within ~2.5 minutes of the block ending,
        // then the slot hash will be unavailable and secure hashes cannot be generated.
        return Err(trace(
            "Slot hash unavailable",
            ProgramError::InvalidAccountData,
        ));
    };
    let slot_hash = slot_hash.to_bytes();
    Ok(slot_hash)
}

// Use slot hash to generate a random u64
fn rng(slot_hash: &[u8]) -> u64 {
    let r1 = u64::from_le_bytes(slot_hash[0..8].try_into().unwrap());
    let r2 = u64::from_le_bytes(slot_hash[8..16].try_into().unwrap());
    let r3 = u64::from_le_bytes(slot_hash[16..24].try_into().unwrap());
    let r4 = u64::from_le_bytes(slot_hash[24..32].try_into().unwrap());
    let r = r1 ^ r2 ^ r3 ^ r4;
    r
}

fn get_winning_square(r: u64) -> usize {
    // Returns a value in the range [0, 25)
    (r % 25) as usize
}

// Returns true if the motherlode was activated, with 1 in 625 chance.
fn is_motherlode_activated(r: u64) -> bool {
    let r = r.reverse_bits();
    let x = r % 625;
    if x == 0 {
        sol_log("Motherlode hit!");
    } else {
        sol_log(&format!("Motherlode missed by {}", x).as_str());
    }
    x == 0
}

/// Randomly selects a winning miner based on their proportional deposits.
/// Uses the slot hash as a source of randomness to select a value between 0 and total_deployed.
/// Each miner's chance of winning is proportional to their deposit amount relative to total deposits.
/// Returns the index of the winning miner, or None if no miners have deposits.
fn get_winning_miner(r: u64, total_deployed: u64, miner_deployed: [u64; 16]) -> Option<usize> {
    // If no miners have deposits, return None.
    if total_deployed == 0 {
        return None;
    }

    // Returns a value in the range [0, total_deployed)
    let r = r.reverse_bits();
    let x = (r % total_deployed) as u64;

    // Find the miner that deposited the winning amount.
    let mut cumulative_deployed = 0;
    for (i, deployed) in miner_deployed.iter().enumerate() {
        cumulative_deployed += *deployed;
        if x < cumulative_deployed {
            return Some(i);
        }
    }

    return None;
}

#[cfg(test)]
mod tests {
    use ore_api::consts::ONE_ORE;
    use solana_program::native_token::sol_to_lamports;

    #[test]
    fn test_get_winning_square() {
        let capacity = 400u64;
        let current = 1000u64;
        let limit = capacity / 4;
        for occupancy in 0..=capacity {
            let availability = capacity.saturating_sub(occupancy);
            if availability == 0 {
                println!("[{}/{}] New: {}", occupancy, capacity, current * 2);
            } else if availability >= limit {
                let pct = (availability * 100) / capacity;
                let chg = (pct.saturating_sub(25) * 100) / 75;
                let dif = (current * chg) / 100;
                let new = current.saturating_sub(dif);
                println!("[{}/{}] New: {}", occupancy, capacity, new);
            } else {
                println!("[{}/{}] New: {}", occupancy, capacity, current);
            }
        }
        // assert!(false);
    }

    #[test]
    fn test_motherlode_math() {
        let motherlode = ONE_ORE * 250;
        let sol_deployed = sol_to_lamports(1.0);
        let total_sol_deploy = sol_deployed * 2;
        let motherlode_winnings =
            ((motherlode as u128 * sol_deployed as u128) / total_sol_deploy as u128) as u64;
        println!("Motherlode winnings: {}", motherlode_winnings);
    }
}
