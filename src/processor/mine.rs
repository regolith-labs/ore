use std::mem::size_of;

use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    keccak::hashv,
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
    DIFFICULTY_RANGE, EPOCH_DURATION,
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
    let [signer, bus_info, config_info, noise_info, proof_info, slot_hashes_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any_bus(bus_info, true)?;
    load_config(config_info, false)?;
    load_noise(noise_info, false)?;
    load_proof(proof_info, signer.key, true)?;
    load_sysvar(slot_hashes_info, sysvar::slot_hashes::id())?;

    // Validate mining is allowed
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
    // let treasury_data = treasury_info.data.borrow();
    // let treasury = Treasury::try_from_bytes(&treasury_data)?;
    // let threshold = treasury.last_reset_at.saturating_add(EPOCH_DURATION);
    // if clock.unix_timestamp.ge(&threshold) {
    //     return Err(OreError::NeedsReset.into());
    // }

    // Calculate the hash from the provided nonce
    let noise_data = noise_info.data.borrow();
    let hx = drillx::hash(&proof.hash, &args.nonce, &noise_data);
    drop(noise_data);

    // Validate hash satisfies the minimnum difficulty
    let difficulty = drillx::difficulty(hx);
    if difficulty.le(&config.min_difficulty) {
        return Err(OreError::DifficultyInsufficient.into());
    }

    // Calculate base reward rate
    let difficulty = difficulty
        .saturating_sub(config.min_difficulty)
        .min(DIFFICULTY_RANGE as u32);
    let mut reward = config
        .base_reward_rate
        .saturating_mul(2u64.saturating_pow(difficulty));

    // Apply staking multiplier
    if clock.slot.gt(&proof.last_deposit_slot) {
        // Only apply if last deposit was at least 1 block ago to prevent flash loan attacks.
        // TODO Cleanup math with a const here (unnecessary cus)
        // TODO Maybe move const into config!?
        let max_stake = reward
            .saturating_mul(60) // min/hour
            .saturating_mul(24) // hour/day
            .saturating_mul(365) // day/year
            .saturating_mul(2); // year
        let staking_reward = proof
            .balance
            .min(max_stake)
            .saturating_mul(reward)
            .saturating_div(max_stake);
        reward = reward.saturating_add(staking_reward);
    }

    // Apply liveness penalty
    // TODO Should penalty be symmetric?
    // TODO Or should the curve be steeper on the <1 min side?
    // TODO Eg anything more frequent than 40 seconds should get 0
    // TODO Anything longer than 2 minutes should be 0
    let tolerance = 5i64; // TODO Get from config
    let target_time = proof.last_hash_at.saturating_add(EPOCH_DURATION);
    if clock
        .unix_timestamp
        .saturating_sub(target_time)
        .abs()
        .gt(&tolerance)
    {
        // TODO Apply
    }

    // Update balances
    let mut bus_data = bus_info.data.borrow_mut();
    let bus = Bus::try_from_bytes_mut(&mut bus_data)?;
    bus.rewards = bus
        .rewards
        .checked_sub(reward)
        .ok_or(OreError::BusRewardsInsufficient)?;
    proof.balance = proof.balance.saturating_add(reward);

    // Hash recent slot hash into the next challenge to prevent pre-mining attacks
    proof.hash = hashv(&[
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
