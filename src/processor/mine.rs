use std::mem::size_of;

use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    keccak::hashv,
    log::sol_log,
    program_error::ProgramError,
    pubkey::Pubkey,
    slot_hashes::SlotHash,
    sysvar::{self, Sysvar},
};

use crate::{
    error::OreError,
    instruction::MineArgs,
    loaders::*,
    state::{Bus, Config, Proof},
    utils::AccountDeserialize,
    MIN_DIFFICULTY, ONE_MINUTE, TWO_YEARS,
};

// TODO Look into tx introspection to require 1 hash per tx

/// Mine is the primary workhorse instruction of the Ore program. Its responsibilities include:
/// 1. Calculate the hash from the provided nonce.
/// 2. Payout rewards based on difficulty, staking multiplier, and liveness penalty.
/// 3. Generate a new challenge for the miner.
/// 4. Update the miner's lifetime stats.
///
/// Safety requirements:
/// - Mine is a permissionless instruction and can be called by any signer.
/// - Can only succeed if mining is not paused.
/// - Can only succeed if the last reset was less than 60 seconds ago.
/// - Can only succeed if the provided hash satisfies the minimum difficulty requirement.
/// - The the provided proof account must be associated with the signer.
/// - The provided bus, config, noise, stake, and slot hash sysvar must be valid.
pub fn process_mine<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = MineArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, bus_info, config_info, proof_info, slot_hashes_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any_bus(bus_info, true)?;
    load_config(config_info, false)?;
    load_proof(proof_info, signer.key, true)?;
    load_sysvar(slot_hashes_info, sysvar::slot_hashes::id())?;

    // Validate mining is not paused
    let config_data = config_info.data.borrow();
    let config = Config::try_from_bytes(&config_data)?;
    if config.paused.ne(&0) {
        return Err(OreError::IsPaused.into());
    }

    // Validate the clock state
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    if clock.unix_timestamp.lt(&proof.last_hash_at) {
        return Err(OreError::ClockInvalid.into());
    }

    // Validate epoch is active
    if clock
        .unix_timestamp
        .ge(&config.last_reset_at.saturating_add(ONE_MINUTE))
    {
        return Err(OreError::NeedsReset.into());
    }

    // Calculate the hash from the provided nonce
    let hx = drillx::hash(&proof.challenge, &args.nonce);

    // Validate hash satisfies the minimnum difficulty
    let difficulty = drillx::difficulty(hx);
    sol_log(&format!("Diff {}", difficulty));
    if difficulty.lt(&MIN_DIFFICULTY) {
        return Err(OreError::HashTooEasy.into());
    }

    // Calculate base reward rate
    let difficulty = difficulty.saturating_sub(MIN_DIFFICULTY);
    let mut reward = config
        .base_reward_rate
        .saturating_mul(2u64.saturating_pow(difficulty));
    sol_log(&format!("Base {}", reward));

    // Apply staking multiplier, only if last deposit was at least 1 block ago to prevent flash loan attacks.
    if clock.slot.gt(&proof.last_deposit_slot) {
        // TODO Move staking requirement into config? Admin adjustable?
        let max_stake = reward.saturating_mul(TWO_YEARS);
        let staking_reward = proof
            .balance
            .min(max_stake)
            .saturating_mul(reward)
            .saturating_div(max_stake);
        sol_log(&format!("Staking {}", staking_reward));
        reward = reward.saturating_add(staking_reward);
    }

    // Apply spam/liveness penalty
    let t = clock.unix_timestamp;
    let t_target = proof.last_hash_at.saturating_add(ONE_MINUTE);
    let t_spam = t_target.saturating_sub(config.tolerance_spam);
    let t_liveness = t_target.saturating_add(config.tolerance_liveness);
    if t.lt(&t_spam) {
        reward = 0;
        sol_log("Spam penalty");
    } else if t.gt(&t_liveness) {
        reward = reward.saturating_sub(
            reward
                .saturating_mul(t.saturating_sub(t_liveness) as u64)
                .saturating_div(
                    t_target
                        .saturating_add(ONE_MINUTE)
                        .saturating_sub(t_liveness) as u64,
                ),
        );
        sol_log(&format!(
            "Liveness penalty ({} sec) {}",
            t.saturating_sub(t_liveness),
            reward,
        ));
    }

    // Set upper bound to whatever is left in the bus
    let mut bus_data = bus_info.data.borrow_mut();
    let bus = Bus::try_from_bytes_mut(&mut bus_data)?;
    let actual_reward = reward.min(bus.rewards);

    // Update balances
    sol_log(&format!("Total {}", reward));
    sol_log(&format!("Bus {}", bus.rewards));
    bus.theoretical_rewards = bus.theoretical_rewards.saturating_add(reward);
    bus.rewards = bus
        .rewards
        .checked_sub(actual_reward)
        .expect("This should not happen");
    proof.balance = proof.balance.saturating_add(actual_reward);

    // Hash recent slot hash into the next challenge to prevent pre-mining attacks
    proof.challenge = hashv(&[
        hx.as_slice(),
        &slot_hashes_info.data.borrow()[0..size_of::<SlotHash>()],
    ])
    .0;

    // Update time trackers
    proof.last_deposit_slot = clock.slot;
    proof.last_hash_at = clock.unix_timestamp;

    // Update lifetime stats
    proof.total_hashes = proof.total_hashes.saturating_add(1);
    proof.total_rewards = proof.total_rewards.saturating_add(reward);

    // Log the mined rewards
    // set_return_data(reward.to_le_bytes().as_slice());

    Ok(())
}
