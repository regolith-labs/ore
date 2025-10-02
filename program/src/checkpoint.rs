use ore_api::prelude::*;
use solana_program::{log::sol_log, rent::Rent};
use steel::*;

/// Checkpoints a miner's rewards.
pub fn process_checkpoint(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, board_info, miner_info, round_info, treasury_info, system_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info.as_account::<Board>(&ore_api::ID)?;
    let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
    // If miner has already checkpointed this round, return.
    if miner.checkpoint_id == miner.round_id {
        return Ok(());
    }
    if round_info.data_is_empty() {
        // If round account is empty, ensure the correct account was provided.
        // This can happen if the miner attempted to checkpoint after the round expired and the account was closed.
        // In this case, the miner forfeits any potential rewards.
        round_info.has_seeds(&[ROUND, &miner.round_id.to_le_bytes()], &ore_api::ID)?;
        miner.checkpoint_id = miner.round_id;
        return Ok(());
    }
    let round = round_info
        .as_account_mut::<Round>(&ore_api::ID)?
        .assert_mut(|r| r.id < board.round_id)? // Ensure round has ended.
        .assert_mut(|r| r.id == miner.round_id)?; // Ensure miner round ID matches the provided round.
    treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Ensure round is not expired.
    if clock.slot >= round.expires_at {
        // In this case, the miner forfeits any potential rewards.
        miner.checkpoint_id = miner.round_id;
        return Ok(());
    }

    // Get the RNG.
    let Some(r) = round.rng() else {
        // If the round has no RNG, no one wins.
        miner.checkpoint_id = miner.round_id;
        return Ok(());
    };

    // Calculate bot fee.
    let mut bot_fee = 0;
    if clock.slot >= round.expires_at - ONE_DAY_SLOTS {
        // The round expires in less than 24h.
        // Anyone may checkpoint this account and collect the bot fee.
        bot_fee = miner.checkpoint_fee;
        miner.checkpoint_fee = 0;
    }

    // Calculate miner rewards.
    let mut rewards_sol = 0;
    let mut rewards_ore = 0;
    let winning_square = round.winning_square(r) as usize;
    if miner.deployed[winning_square] > 0 {
        // Sanity check.
        assert!(
            round.deployed[winning_square] >= miner.deployed[winning_square],
            "Invalid round deployed amount"
        );

        // Calculate SOL rewards.
        let original_deployment = miner.deployed[winning_square];
        let admin_fee = original_deployment / 100;
        rewards_sol = original_deployment - admin_fee;
        rewards_sol += ((round.total_winnings as u128 * miner.deployed[winning_square] as u128)
            / round.deployed[winning_square] as u128) as u64;

        // Calculate ORE rewards.
        let top_miner_sample = round.top_miner_sample(r, winning_square);
        if top_miner_sample >= miner.cumulative[winning_square]
            && top_miner_sample < miner.cumulative[winning_square] + miner.deployed[winning_square]
        {
            rewards_ore = round.top_miner_reward;
        }

        // Calculate motherlode rewards.
        if round.motherlode > 0 {
            rewards_ore += ((round.motherlode as u128 * miner.deployed[winning_square] as u128)
                / round.deployed[winning_square] as u128) as u64;
        }
    }

    sol_log("I");

    // Checkpoint miner.
    miner.checkpoint_id = round.id;
    miner.rewards_ore += rewards_ore;
    miner.lifetime_rewards_ore += rewards_ore;
    miner.rewards_sol += rewards_sol;
    miner.lifetime_rewards_sol += rewards_sol;

    // Do SOL transfers.
    if rewards_sol > 0 {
        round_info.send(rewards_sol, &miner_info);
    }
    if bot_fee > 0 {
        miner_info.send(bot_fee, &signer_info);
    }

    // TODO Round debts. Track total checkpointed and/or num miner's checkpointed.

    // Assert round has sufficient funds for rent + debts.
    let account_size = 8 + std::mem::size_of::<Round>();
    let required_rent = Rent::get()?.minimum_balance(account_size);
    assert!(
        round_info.lamports() >= required_rent,
        "Round does not have sufficient funds for rent"
    );

    let account_size = 8 + std::mem::size_of::<Miner>();
    let required_rent = Rent::get()?.minimum_balance(account_size);
    assert!(
        miner_info.lamports() >= required_rent + miner.checkpoint_fee + miner.rewards_sol,
        "Miner does not have sufficient funds for rent and rewards"
    );

    Ok(())
}
