use ore_api::prelude::*;
use solana_program::{log::sol_log, slot_hashes::SlotHashes};
use steel::*;

/// Claims a block reward.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let (required_accounts, miner_accounts) = accounts.split_at(9);
    let [signer_info, board_info, mint_info, reserve_tokens_info, treasury_info, treasury_tokens_info, system_program, token_program, slot_hashes_sysvar] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| b.slot_hash == [0; 32])?
        .assert_mut(|b| clock.slot >= b.end_slot)?;
    let mint = mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    reserve_tokens_info
        .has_address(&BOOST_RESERVE_TOKEN)?
        .as_token_account()?
        .assert(|t| t.mint() == MINT_ADDRESS)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info.as_associated_token_account(&treasury_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    sol_log("A");

    // Mint tokens to the boost reserve.
    mint_to_signed(
        mint_info,
        reserve_tokens_info,
        treasury_info,
        token_program,
        ONE_ORE / 3,
        &[TREASURY],
    )?;

    sol_log("B");

    // Sample slot hashes.
    let (winning_square, square_commits) =
        if let Ok(slot_hash) = get_slot_hash(board.end_slot, slot_hashes_sysvar) {
            board.slot_hash = slot_hash;
            let winning_square = get_winning_square(&slot_hash);
            let square_commits = board.commits[winning_square as usize];
            (winning_square, square_commits)
        } else {
            // Cannot get slot hash. No one wins.
            board.slot_hash = [u8::MAX; 32];
            (u64::MAX, 0)
        };

    sol_log("C");

    // No one won. Vault all prospects.
    if square_commits == 0 {
        board.total_vaulted = board.total_prospects;
        treasury.balance += board.total_prospects;
        board_info.send(board.total_prospects, &treasury_info);
        return Ok(());
    }

    sol_log("D");

    // Get winnings amount (prospects on all non-winning squares).
    let mut winnings = 0;
    for (i, commits) in board.commits.iter().enumerate() {
        if i as u64 != winning_square {
            winnings += commits;
        }
    }

    sol_log("E");

    // Get vault amount.
    let vault_amount = winnings / 10; // Vault 10% of winnings.
    board.total_vaulted = vault_amount;
    let winnings = winnings - vault_amount;
    // board_info.send(vault_amount, &treasury_info);
    treasury.balance += vault_amount;

    sol_log("F");

    // Payout winnings to miners.
    let mut top_winner = None;
    let mut top_winner_commits = 0;
    let mut rewards_sol = [0; 16];
    let mut checksum = 0;
    for (i, miner_info) in miner_accounts.iter().enumerate() {
        sol_log("G");
        // Transfer winnings to miner.
        let miner = miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.round_id == board.id)?;
        let miner_commits = miner.commits[winning_square as usize];
        let rewards = miner_commits + (winnings * miner_commits / square_commits); // Winners get their own prospect back plus their share of the winnings.
        miner.rewards_sol += rewards;
        miner.lifetime_rewards_sol += rewards;
        checksum += miner_commits;
        rewards_sol[i] = rewards;
        // board_info.send(rewards, &miner_info);

        // Find the top winner.
        if miner_commits > top_winner_commits {
            sol_log("H");
            top_winner_commits = miner_commits;
            top_winner = Some(i);
            // top_winner = Some(miner);
        }
    }

    sol_log("I");

    // Verify checksum.
    if checksum != square_commits {
        // This can only happen if the caller didn't provide full set of winning miners.
        sol_log("J");
        return Err(ProgramError::InvalidAccountData);
    }

    sol_log("K");

    // Payout reward to top winner.
    if let Some(i) = top_winner {
        sol_log("L");
        let miner = miner_accounts[i].as_account_mut::<Miner>(&ore_api::ID)?;
        let mint_amount = ONE_ORE.min(MAX_SUPPLY - mint.supply());
        if mint_amount > 0 {
            // sol_log("M");
            miner.rewards_ore += mint_amount;
            miner.lifetime_rewards_ore += mint_amount;
            board.top_winner = miner.authority;
            // sol_log("M2");
            mint_to_signed(
                mint_info,
                treasury_tokens_info,
                treasury_info,
                token_program,
                mint_amount,
                &[TREASURY],
            )?;
            // sol_log("N");
        }

        // miner.rewards_ore += rewards_sol[i];
        // miner.lifetime_rewards_ore += rewards_sol[i];
        // board.top_winner = miner.authority;
    }

    // if let Some(miner) = top_winner {
    //     sol_log("L");
    //     let mint_amount = ONE_ORE.min(MAX_SUPPLY - mint.supply());
    //     sol_log(&format!("mint_amount: {}", mint_amount));
    //     if mint_amount > 0 {
    //         sol_log("M");
    //         miner.rewards_ore += mint_amount;
    //         miner.lifetime_rewards_ore += mint_amount;
    //         board.top_winner = miner.authority;
    //         sol_log("M2");
    //         mint_to_signed(
    //             mint_info,
    //             treasury_tokens_info,
    //             treasury_info,
    //             token_program,
    //             mint_amount,
    //             &[TREASURY],
    //         )?;
    //         sol_log("N");
    //     }
    // }

    sol_log("O");

    // Update board.
    board.total_winnings = winnings;

    // Do SOL transfers.
    board_info.send(vault_amount, &treasury_info);
    for (i, miner_info) in miner_accounts.iter().enumerate() {
        board_info.send(rewards_sol[i], &miner_info);
    }

    // Send vault amount to treasury.

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
        return Err(ProgramError::InvalidAccountData);
    };
    let slot_hash = slot_hash.to_bytes();
    Ok(slot_hash)
}

fn get_winning_square(slot_hash: &[u8]) -> u64 {
    // Use slot hash to generate a random u64
    let r1 = u64::from_le_bytes(slot_hash[0..8].try_into().unwrap());
    let r2 = u64::from_le_bytes(slot_hash[8..16].try_into().unwrap());
    let r3 = u64::from_le_bytes(slot_hash[16..24].try_into().unwrap());
    let r4 = u64::from_le_bytes(slot_hash[24..32].try_into().unwrap());
    let r = r1 ^ r2 ^ r3 ^ r4;

    // Returns a value in the range [0, 24] inclusive
    r % 25
}
