use ore_api::prelude::*;
use solana_program::rent::Rent;
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

    // If round account is empty, verify the correct account was provided.
    // This can happen if the miner attempted to checkpoint after the round expired and the account was closed.
    // In this case, the miner forfeits any potential rewards.
    if round_info.data_is_empty() {
        round_info.has_seeds(&[ROUND, &miner.round_id.to_le_bytes()], &ore_api::ID)?;
        miner.checkpoint_id = miner.round_id;
        return Ok(());
    }

    let round = round_info.as_account_mut::<Round>(&ore_api::ID)?; // Round has been closed.
    treasury_info.as_account::<Treasury>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // If round is current round, or the miner round ID does not match the provided round, return.
    if round.id == board.round_id || round.id != miner.round_id || round.slot_hash == [0; 32] {
        return Ok(());
    }

    // Ensure round is not expired.
    // In this case, the miner forfeits any potential rewards.
    if clock.slot >= round.expires_at {
        miner.checkpoint_id = miner.round_id;
        return Ok(());
    }

    // Calculate bot fee.
    // If the round expires in less than 12h, anyone may checkpoint this account and collect the bot fee.
    let mut bot_fee = 0;
    if clock.slot >= round.expires_at - TWELVE_HOURS_SLOTS {
        bot_fee = miner.checkpoint_fee;
        miner.checkpoint_fee = 0;
    }

    // Calculate miner rewards.
    let mut rewards_sol = 0;
    let mut rewards_ore = 0;

    // Get the RNG.
    if let Some(r) = round.rng() {
        // Get the winning square.
        let winning_square = round.winning_square(r) as usize;

        // If the miner deployed to the winning square, calculate rewards.
        if miner.deployed[winning_square] > 0 {
            // Sanity check.
            assert!(
                round.deployed[winning_square] >= miner.deployed[winning_square],
                "Invalid round deployed amount"
            );

            // Calculate SOL rewards.
            let original_deployment = miner.deployed[winning_square];
            let admin_fee = (original_deployment / 100).max(1);
            rewards_sol = original_deployment - admin_fee;
            rewards_sol += ((round.total_winnings as u128 * miner.deployed[winning_square] as u128)
                / round.deployed[winning_square] as u128) as u64;

            // Calculate ORE rewards.
            if round.top_miner == SPLIT_ADDRESS {
                // If round is split, split the reward evenly among all miners.
                rewards_ore = ((round.top_miner_reward * miner.deployed[winning_square])
                    / round.deployed[winning_square]) as u64;
            } else {
                // If round is not split, payout to the top miner.
                let top_miner_sample = round.top_miner_sample(r, winning_square);
                if top_miner_sample >= miner.cumulative[winning_square]
                    && top_miner_sample
                        < miner.cumulative[winning_square] + miner.deployed[winning_square]
                {
                    rewards_ore = round.top_miner_reward;
                    round.top_miner = miner.authority;
                }
            }

            // Calculate motherlode rewards.
            if round.motherlode > 0 {
                rewards_ore += ((round.motherlode as u128 * miner.deployed[winning_square] as u128)
                    / round.deployed[winning_square] as u128) as u64;
            }
        }
    } else {
        // Sanity check.
        // If there is no rng, total deployed should have been reset to zero.
        assert!(
            round.total_deployed == 0,
            "Round total deployed should be zero."
        );

        // Round has no slot hash, refund all SOL.
        rewards_sol = miner.deployed.iter().sum::<u64>();
    }

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

    // Assert miner account has sufficient funds for rent and rewards.
    let account_size = 8 + std::mem::size_of::<Miner>();
    let required_rent = Rent::get()?.minimum_balance(account_size);
    assert!(
        miner_info.lamports() >= required_rent + miner.checkpoint_fee + miner.rewards_sol,
        "Miner does not have sufficient funds for rent and rewards"
    );

    Ok(())
}
