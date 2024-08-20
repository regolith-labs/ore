use std::mem::size_of;

use drillx::Solution;
use ore_api::{
    consts::*,
    error::OreError,
    event::MineEvent,
    instruction::MineArgs,
    loaders::*,
    state::{Bus, Config, Proof},
};
use solana_program::program::set_return_data;
#[allow(deprecated)]
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    keccak::hashv,
    program_error::ProgramError,
    pubkey::Pubkey,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16},
    slot_hashes::SlotHash,
    sysvar::{self, Sysvar},
};

use crate::utils::AccountDeserialize;

/// Mine validates hashes and increments a miner's collectable balance.
pub fn process_mine<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = MineArgs::try_from_bytes(data)?;

    // Load accounts.
    let [signer, bus_info, config_info, proof_info, instructions_sysvar, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any_bus(bus_info, true)?;
    load_config(config_info, false)?;
    load_proof_with_miner(proof_info, signer.key, true)?;
    load_sysvar(instructions_sysvar, sysvar::instructions::id())?;
    load_sysvar(slot_hashes_sysvar, sysvar::slot_hashes::id())?;

    // Authenticate the proof account.
    //
    // Only one proof account can be used for any given transaction. All `mine` instructions
    // in the transaction must use the same proof account.
    authenticate(&instructions_sysvar.data.borrow(), proof_info.key)?;

    // Validate epoch is active.
    let config_data = config_info.data.borrow();
    let config = Config::try_from_bytes(&config_data)?;
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    if config
        .last_reset_at
        .saturating_add(EPOCH_DURATION)
        .le(&clock.unix_timestamp)
    {
        return Err(OreError::NeedsReset.into());
    }

    // Validate the hash digest.
    //
    // Here we use drillx to validate the provided solution is a valid hash of the challenge.
    // If invalid, we return an error.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    let solution = Solution::new(args.digest, args.nonce);
    if !solution.is_valid(&proof.challenge) {
        return Err(OreError::HashInvalid.into());
    }

    // Reject spam transactions.
    //
    // If a miner attempts to submit solutions too frequently, we reject with an error. In general,
    // miners are limited to 1 hash per epoch on average.
    let t: i64 = clock.unix_timestamp;
    let t_target = proof.last_hash_at.saturating_add(ONE_MINUTE);
    let t_spam = t_target.saturating_sub(TOLERANCE);
    if t.lt(&t_spam) {
        return Err(OreError::Spam.into());
    }

    // Validate the hash satisfies the minimum difficulty.
    //
    // We use drillx to get the difficulty (leading zeros) of the hash. If the hash does not have the
    // minimum required difficulty, we reject it with an error.
    let hash = solution.to_hash();
    let difficulty = hash.difficulty();
    if difficulty.lt(&(config.min_difficulty as u32)) {
        return Err(OreError::HashTooEasy.into());
    }

    // Normalize the difficulty and calculate the reward amount.
    //
    // The reward doubles for every bit of difficulty (leading zeros) on the hash. We use the normalized
    // difficulty so the minimum accepted difficulty pays out at the base reward rate.
    let normalized_difficulty = difficulty
        .checked_sub(config.min_difficulty as u32)
        .unwrap();
    let mut reward = config
        .base_reward_rate
        .checked_mul(2u64.checked_pow(normalized_difficulty).unwrap())
        .unwrap();

    // Apply staking multiplier.
    //
    // If user has greater than or equal to the max stake on the network, they receive 2x multiplier.
    // Any stake less than this will receives between 1x and 2x multipler. The multipler is only active
    // if the miner's last stake deposit was more than one minute ago to protect against flash loan attacks.
    let mut bus_data = bus_info.data.borrow_mut();
    let bus = Bus::try_from_bytes_mut(&mut bus_data)?;
    if proof.balance.gt(&0) && proof.last_stake_at.saturating_add(ONE_MINUTE).lt(&t) {
        // Calculate staking reward.
        if config.top_balance.gt(&0) {
            let staking_reward = (reward as u128)
                .checked_mul(proof.balance.min(config.top_balance) as u128)
                .unwrap()
                .checked_div(config.top_balance as u128)
                .unwrap() as u64;
            reward = reward.checked_add(staking_reward).unwrap();
        }

        // Update bus stake tracker.
        if proof.balance.gt(&bus.top_balance) {
            bus.top_balance = proof.balance;
        }
    }

    // Apply liveness penalty.
    //
    // The liveness penalty exists to ensure there is no "invisible" hashpower on the network. It
    // should not be possible to spend ~1 hour on a given challenge and submit a hash with a large
    // difficulty value to earn an outsized reward.
    //
    // The penalty works by halving the reward amount for every minute late the solution has been submitted.
    // This ultimately drives the reward to zero given enough time (10-20 minutes).
    let t_liveness = t_target.saturating_add(TOLERANCE);
    if t.gt(&t_liveness) {
        // Halve the reward for every minute late.
        let tardiness = t.saturating_sub(t_target) as u64;
        let halvings = tardiness.saturating_div(ONE_MINUTE as u64);
        if halvings.gt(&0) {
            reward = reward.saturating_div(2u64.saturating_pow(halvings as u32));
        }

        // Linear decay with remainder seconds.
        let remainder_secs = tardiness.saturating_sub(halvings.saturating_mul(ONE_MINUTE as u64));
        if remainder_secs.gt(&0) && reward.gt(&0) {
            let penalty = reward
                .saturating_div(2)
                .saturating_mul(remainder_secs)
                .saturating_div(ONE_MINUTE as u64);
            reward = reward.saturating_sub(penalty);
        }
    }

    // Limit payout amount to whatever is left in the bus.
    //
    // Busses are limited to distributing 1 ORE per epoch. This is also the maximum amount that will be paid out
    // for any given hash.
    let reward_actual = reward.min(bus.rewards).min(ONE_ORE);

    // Update balances.
    //
    // We track the theoretical rewards that would have been paid out ignoring the bus limit, so the
    // base reward rate will be updated to account for the real hashpower on the network.
    bus.theoretical_rewards = bus.theoretical_rewards.checked_add(reward).unwrap();
    bus.rewards = bus.rewards.checked_sub(reward_actual).unwrap();
    proof.balance = proof.balance.checked_add(reward_actual).unwrap();

    // Hash a recent slot hash into the next challenge to prevent pre-mining attacks.
    //
    // The slot hashes are unpredictable values. By seeding the next challenge with the most recent slot hash,
    // miners are forced to submit their current solution before they can begin mining for the next.
    proof.last_hash = hash.h;
    proof.challenge = hashv(&[
        hash.h.as_slice(),
        &slot_hashes_sysvar.data.borrow()[0..size_of::<SlotHash>()],
    ])
    .0;

    // Update time trackers.
    proof.last_hash_at = t.max(t_target);

    // Update lifetime stats.
    proof.total_hashes = proof.total_hashes.saturating_add(1);
    proof.total_rewards = proof.total_rewards.saturating_add(reward);

    // Log the mined rewards.
    //
    // This data can be used by off-chain indexers to display mining stats.
    set_return_data(
        MineEvent {
            difficulty: difficulty as u64,
            reward: reward_actual,
            timing: t.saturating_sub(t_liveness),
        }
        .to_bytes(),
    );

    Ok(())
}

/// Authenticate the proof account.
///
/// This process is necessary to prevent sybil attacks. If a user can pack multiple hashes into a single
/// transaction, then there is a financial incentive to mine across multiple keypairs and submit as many hashes
/// as possible in the same transaction to minimize fee / hash.
///
/// This is prevented by forcing every transaction to declare upfront the proof account that will be used for mining.
/// The authentication process includes passing the 32 byte pubkey address as instruction data to a CU-optimized noop
/// program. We parse this address through transaction introspection and use it to ensure the same proof account is
/// used for every `mine` instruction in a given transaction.
fn authenticate(data: &[u8], proof_address: &Pubkey) -> ProgramResult {
    if let Ok(Some(auth_address)) = parse_auth_address(data) {
        if proof_address.ne(&auth_address) {
            return Err(OreError::AuthFailed.into());
        }
    } else {
        return Err(OreError::AuthFailed.into());
    }
    Ok(())
}

/// Use transaction introspection to parse the authenticated pubkey.
fn parse_auth_address(data: &[u8]) -> Result<Option<Pubkey>, SanitizeError> {
    // Start the current byte index at 0
    let mut curr = 0;
    let num_instructions = read_u16(&mut curr, data)?;
    let pc = curr;

    // Iterate through the transaction instructions
    for i in 0..num_instructions as usize {
        // Shift pointer to correct positition
        curr = pc + i * 2;
        curr = read_u16(&mut curr, data)? as usize;

        // Skip accounts
        let num_accounts = read_u16(&mut curr, data)? as usize;
        curr += num_accounts * 33;

        // Read the instruction program id
        let program_id = read_pubkey(&mut curr, data)?;

        // Introspect on the first noop instruction
        if program_id.eq(&NOOP_PROGRAM_ID) {
            // Retrun address read from instruction data
            curr += 2;
            let address = read_pubkey(&mut curr, data)?;
            return Ok(Some(address));
        }
    }

    // Default return none
    Ok(None)
}
