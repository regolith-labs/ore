use std::mem::size_of;

use drillx::Solution;
use ore_api::prelude::*;
use ore_boost_api::{
    consts::{DENOMINATOR_MULTIPLIER, ROTATION_DURATION},
    state::{Boost, Config as BoostConfig},
};
use solana_program::{
    keccak::hashv,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16},
    slot_hashes::SlotHash,
};
use steel::*;

/// Mine validates hashes and increments a miner's claimable balance.
pub fn process_mine(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Mine::try_from_bytes(data)?;

    // Load accounts.
    let clock = Clock::get()?;
    let t: i64 = clock.unix_timestamp;
    let (required_accounts, optional_accounts) = accounts.split_at(6);
    let [signer_info, bus_info, config_info, proof_info, instructions_sysvar, slot_hashes_sysvar] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let bus = bus_info.is_bus()?.as_account_mut::<Bus>(&ore_api::ID)?;
    let config = config_info
        .is_config()?
        .as_account::<Config>(&ore_api::ID)?
        .assert_err(
            |c| c.last_reset_at.saturating_add(EPOCH_DURATION) > t,
            OreError::NeedsReset.into(),
        )?;
    let proof = proof_info
        .as_account_mut::<Proof>(&ore_api::ID)?
        .assert_mut_err(
            |p| p.miner == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?;
    instructions_sysvar.is_sysvar(&sysvar::instructions::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Authenticate the proof account.
    //
    // Only one proof account can be used for any given transaction. All `mine` instructions
    // in the transaction must use the same proof account.
    authenticate(&instructions_sysvar.data.borrow(), proof_info.key)?;

    // Reject spam transactions.
    //
    // Miners are rate limited to approximately 1 hash per minute. If a miner attempts to submit
    // solutions more frequently than this, reject with an error.
    let t_target = proof.last_hash_at.saturating_add(ONE_MINUTE);
    let t_spam = t_target.saturating_sub(TOLERANCE);
    if t.lt(&t_spam) {
        return Err(OreError::Spam.into());
    }

    // Validate the hash digest.
    //
    // Here we use drillx to validate the provided solution is a valid hash of the challenge.
    // If invalid, we return an error.
    let solution = Solution::new(args.digest, args.nonce);
    if !solution.is_valid(&proof.challenge) {
        return Err(OreError::HashInvalid.into());
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
    let base_reward = config
        .base_reward_rate
        .checked_mul(2u64.checked_pow(normalized_difficulty).unwrap())
        .unwrap();

    // Apply boosts.
    //
    // Boosts are staking incentives that can multiply a miner's rewards. The boost rewards are
    // split between the miner and staker.
    let mut boost_reward = 0;
    if let [boost_info, _boost_proof_info, boost_config_info] = optional_accounts {
        // Load boost accounts.
        let boost = boost_info.as_account::<Boost>(&ore_boost_api::ID)?;
        let boost_config = boost_config_info.as_account::<BoostConfig>(&ore_boost_api::ID)?;

        // Apply multiplier if boost is active, not expired, and last rotation was less than one minute ago
        if boost_config.current == *boost_info.key
            && t < boost_config.ts + ROTATION_DURATION
            && t < boost.expires_at
        {
            boost_reward = (base_reward as u128)
                .checked_mul(boost.multiplier as u128)
                .unwrap()
                .checked_div(DENOMINATOR_MULTIPLIER as u128)
                .unwrap() as u64;
        }
    }

    // Apply liveness penalty.
    //
    // The liveness penalty exists to ensure there is no "invisible" hashpower on the network. It
    // should not be possible to spend ~1 hour on a given challenge and submit a hash with a large
    // difficulty value to earn an outsized reward.
    //
    // The penalty works by decaying the reward to zero over a 60 second period after the target time.
    let gross_reward = base_reward.checked_add(boost_reward).unwrap();
    let mut gross_penalized_reward = gross_reward;
    let t_liveness = t_target.saturating_add(TOLERANCE);
    if t > t_liveness {
        let secs_late = t.saturating_sub(t_target).min(ONE_MINUTE) as u64;
        let penalty = gross_reward
            .saturating_mul(secs_late)
            .saturating_div(ONE_MINUTE as u64);
        gross_penalized_reward = gross_penalized_reward.saturating_sub(penalty);
    }

    // Apply bus limit.
    //
    // Busses are limited to distributing 1 ORE per epoch. The payout amount must be capped to whatever is
    // left in the selected bus. This limits the maximum amount that will be paid out for any given hash to 1 ORE.
    let net_reward = gross_penalized_reward.min(bus.rewards).min(ONE_ORE);

    // Scale the base and boost rewards to account for penalties.
    let net_base_reward = if gross_reward > 0 {
        (net_reward as u128)
            .checked_mul(base_reward as u128)
            .unwrap()
            .checked_div(gross_reward as u128)
            .unwrap() as u64
    } else {
        0
    };
    let net_boost_reward = net_reward.checked_sub(net_base_reward).unwrap();

    // Split the boost rewards between miner and staker.
    let net_staker_boost_reward = net_boost_reward.checked_div(2).unwrap();
    let net_miner_boost_reward = net_boost_reward
        .checked_sub(net_staker_boost_reward)
        .unwrap();
    let net_miner_reward = net_base_reward.checked_add(net_miner_boost_reward).unwrap();

    // Checksum on rewards. Should never fail.
    assert!(
        net_reward
            == net_base_reward
                .checked_add(net_miner_boost_reward)
                .unwrap()
                .checked_add(net_staker_boost_reward)
                .unwrap(),
        "Rewards checksum failed"
    );

    // Update bus balances.
    //
    // We track the theoretical rewards that would have been paid out ignoring the bus limit, so the
    // base reward rate will be updated to account for the real hashpower on the network.
    bus.theoretical_rewards = bus
        .theoretical_rewards
        .checked_add(gross_penalized_reward)
        .unwrap();
    bus.rewards = bus.rewards.checked_sub(net_reward).unwrap();

    // Update miner balances.
    proof.balance = proof.balance.checked_add(net_miner_reward).unwrap();

    // Update staker balances.
    if net_staker_boost_reward > 0 {
        if let [boost_info, boost_proof_info, _boost_config_info] = optional_accounts {
            let boost_proof = boost_proof_info
                .as_account_mut::<Proof>(&ore_api::ID)?
                .assert_mut(|p| p.authority == *boost_info.key)?;
            boost_proof.balance = boost_proof
                .balance
                .checked_add(net_staker_boost_reward)
                .unwrap();
            boost_proof.total_rewards = boost_proof
                .total_rewards
                .checked_add(net_staker_boost_reward)
                .unwrap();
        }
    }

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

    // Update stats.
    let prev_last_hash_at = proof.last_hash_at;
    proof.last_hash_at = t.max(t_target);
    proof.total_hashes = proof.total_hashes.saturating_add(1);
    proof.total_rewards = proof.total_rewards.saturating_add(net_miner_reward);

    // Log data.
    //
    // The boost rewards are scaled down before logging to account for penalties and bus limits.
    // This return data can be used by pool operators to calculate miner and staker rewards.
    MineEvent {
        balance: proof.balance,
        difficulty: difficulty as u64,
        last_hash_at: prev_last_hash_at,
        timing: t.saturating_sub(t_liveness),
        net_reward,
        net_base_reward,
        net_miner_boost_reward,
        net_staker_boost_reward,
    }
    .log_return();

    Ok(())
}

/// Authenticate the proof account.
///
/// This process is necessary to prevent sybil attacks. If a user can pack multiple hashes into a single
/// transaction, then there is a financial incentive to mine across multiple keypairs and submit as many hashes
/// as possible in the same transaction to minimize fee / hash.
///
/// We prevent this by forcing every transaction to declare upfront the proof account that will be used for mining.
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
            // Return address read from instruction data
            curr += 2;
            let address = read_pubkey(&mut curr, data)?;
            return Ok(Some(address));
        }
    }

    // Default return none
    Ok(None)
}
