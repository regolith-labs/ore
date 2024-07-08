use std::mem::size_of;

use drillx::Solution;
use ore_api::{
    consts::*,
    error::OreError,
    event::MineEvent,
    instruction::{MineArgs, OreInstruction},
    loaders::*,
    state::{Bus, Config, Proof},
};
use solana_program::program::set_return_data;
#[allow(deprecated)]
use solana_program::{
    account_info::AccountInfo,
    blake3::hashv,
    clock::Clock,
    entrypoint::ProgramResult,
    log::sol_log,
    program_error::ProgramError,
    pubkey::Pubkey,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16, read_u8},
    slot_hashes::SlotHash,
    sysvar::{self, instructions::load_current_index, Sysvar},
};

use crate::utils::AccountDeserialize;

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
/// - The provided proof account must be associated with the signer.
/// - The provided bus, config, noise, stake, and slot hash sysvar must be valid.
pub fn process_mine<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = MineArgs::try_from_bytes(data)?;

    // Load accounts
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

    // Validate this is the only mine ix in the transaction.
    if !introspect_transaction(&instructions_sysvar.data.borrow()).unwrap_or(false) {
        return Err(OreError::TransactionInvalid.into());
    }

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
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    let solution = Solution::new(args.digest, args.nonce);
    if !solution.is_valid(&proof.challenge) {
        return Err(OreError::HashInvalid.into());
    }

    // Validate hash satisfies the minimnum difficulty.
    let hash = solution.to_hash();
    let difficulty = hash.difficulty();
    sol_log(&format!("Diff {}", difficulty));
    if difficulty.lt(&MIN_DIFFICULTY) {
        return Err(OreError::HashTooEasy.into());
    }
    let mut reward = config
        .base_reward_rate
        .checked_mul(2u64.checked_pow(difficulty).unwrap())
        .unwrap();

    // Apply staking multiplier.
    // If user has greater than or equal to the max stake on the network, they receive 2x multiplier.
    // Any stake less than this will receives between 1x and 2x multipler. The multipler is only active
    // if the miner's last stake deposit was more than one minute ago.
    let t = clock.unix_timestamp;
    if config.max_stake.gt(&0) && proof.last_stake_at.saturating_add(ONE_MINUTE).le(&t) {
        let staking_reward = proof
            .balance
            .min(config.max_stake)
            .checked_mul(reward)
            .unwrap()
            .checked_div(config.max_stake)
            .unwrap();
        reward = reward.checked_add(staking_reward).unwrap();
    }

    // Reject spam transactions.
    let t_target = proof.last_hash_at.saturating_add(ONE_MINUTE);
    let t_spam = t_target.saturating_sub(TOLERANCE);
    if t.lt(&t_spam) {
        return Err(OreError::Spam.into());
    }

    // Apply liveness penalty.
    let t_liveness = t_target.saturating_add(TOLERANCE);
    if t.gt(&t_liveness) {
        reward = reward
            .checked_sub(
                reward
                    .checked_mul(t.checked_sub(t_liveness).unwrap() as u64)
                    .unwrap()
                    .checked_div(ONE_MINUTE as u64)
                    .unwrap(),
            )
            .unwrap();
    }

    // Limit payout amount to whatever is left in the bus
    let mut bus_data = bus_info.data.borrow_mut();
    let bus = Bus::try_from_bytes_mut(&mut bus_data)?;
    let reward_actual = reward.min(bus.rewards);

    // Update balances
    bus.theoretical_rewards = bus.theoretical_rewards.checked_add(reward).unwrap();
    bus.rewards = bus.rewards.checked_sub(reward_actual).unwrap();
    proof.balance = proof.balance.checked_add(reward_actual).unwrap();

    // Hash recent slot hash into the next challenge to prevent pre-mining attacks
    proof.last_hash = hash.h;
    proof.challenge = hashv(&[
        hash.h.as_slice(),
        &slot_hashes_sysvar.data.borrow()[0..size_of::<SlotHash>()],
    ])
    .0;

    // Update time trackers
    proof.last_hash_at = t.max(t_target);

    // Update lifetime stats
    proof.total_hashes = proof.total_hashes.saturating_add(1);
    proof.total_rewards = proof.total_rewards.saturating_add(reward);

    // Log the mined rewards
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

/// Require that there is only one `mine` instruction per transaction and it is called from the
/// top level of the transaction.
///
/// The intent here is to disincentivize sybil. As long as a user can fit multiple hashes in a single
/// transaction, there is a financial incentive to sybil multiple keypairs and pack as many hashes
/// as possible into each transaction to minimize fee / hash.
///
/// If each transaction is limited to one hash only, then a user will minimize their fee / hash
/// by allocating all their hashpower to finding the single most difficult hash they can.
fn introspect_transaction(msg: &[u8]) -> Result<bool, SanitizeError> {
    #[allow(deprecated)]
    let idx = load_current_index(msg);
    let mut c = 0;
    let num_instructions = read_u16(&mut c, msg)?;
    let pc = c;
    for i in 0..num_instructions as usize {
        c = pc + i * 2;
        c = read_u16(&mut c, msg)? as usize;
        let num_accounts = read_u16(&mut c, msg)? as usize;
        c += num_accounts * 33;
        let program_id = read_pubkey(&mut c, msg)?;
        if i.eq(&(idx as usize)) {
            // Require top-level instruction at current index is a `mine`
            if program_id.ne(&ore_api::ID) {
                return Ok(false);
            }
            if let Ok(ix) = OreInstruction::try_from(read_u8(&mut c, msg)?) {
                if ix.ne(&OreInstruction::Mine) {
                    return Ok(false);
                }
            }
        } else {
            // Require no other instructions in the transaction are a `mine`
            if program_id.eq(&ore_api::ID) {
                c += 2;
                if let Ok(ix) = OreInstruction::try_from(read_u8(&mut c, msg)?) {
                    if ix.eq(&OreInstruction::Mine) {
                        return Ok(false);
                    }
                }
            }
        }
    }

    Ok(true)
}
