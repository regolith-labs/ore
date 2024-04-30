use std::mem::size_of;

#[allow(deprecated)]
use solana_program::{
    account_info::AccountInfo,
    clock::Clock,
    entrypoint::ProgramResult,
    keccak::hashv,
    log::sol_log,
    program_error::ProgramError,
    pubkey::Pubkey,
    sanitize::SanitizeError,
    serialize_utils::{read_pubkey, read_u16, read_u8},
    slot_hashes::SlotHash,
    sysvar::{self, instructions::load_current_index, Sysvar},
};

use crate::{
    error::OreError,
    instruction::{MineArgs, OreInstruction},
    loaders::*,
    state::{Bus, Config, Proof},
    utils::AccountDeserialize,
    COMPUTE_BUDGET_PROGRAM_ID, MIN_DIFFICULTY, ONE_MINUTE, TWO_YEARS,
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
    let [signer, bus_info, config_info, proof_info, instructions_sysvar, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any_bus(bus_info, true)?;
    load_config(config_info, false)?;
    load_proof(proof_info, signer.key, true)?;
    load_sysvar(instructions_sysvar, sysvar::instructions::id())?;
    load_sysvar(slot_hashes_sysvar, sysvar::slot_hashes::id())?;

    // Validate this is the only mine ix in the transaction
    if !validate_transaction(&instructions_sysvar.data.borrow()).unwrap_or(false) {
        return Err(OreError::TransactionInvalid.into());
    }

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

    // Apply staking multiplier.
    // To prevent flash loan attacks, multiplier is only applied if the last deposit was at least 1 block ago.
    // The multiplier can range 1x to 2x. To get the maximum multiplier, the stake balance must be
    // greater than or equal to two years worth of rewards at the selected difficulty.
    if clock.slot.gt(&proof.last_deposit_slot) {
        let upper_bound = reward.saturating_mul(TWO_YEARS);
        let staking_reward = proof
            .balance
            .min(upper_bound)
            .saturating_mul(reward)
            .saturating_div(upper_bound);
        reward = reward.saturating_add(staking_reward);
        sol_log(&format!("Staking {}", staking_reward));
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
        &slot_hashes_sysvar.data.borrow()[0..size_of::<SlotHash>()],
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

/// Require that there is only one `mine` instruction per transaction and it is called from the
/// top level of the transaction.
fn validate_transaction(msg: &[u8]) -> Result<bool, SanitizeError> {
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
        match read_pubkey(&mut c, msg)? {
            crate::ID => {
                c += 2;
                if let Ok(ix) = OreInstruction::try_from(read_u8(&mut c, msg)?) {
                    if let OreInstruction::Mine = ix {
                        if i.ne(&(idx as usize)) {
                            return Ok(false);
                        }
                    }
                } else {
                    return Ok(false);
                }
            }
            COMPUTE_BUDGET_PROGRAM_ID => {
                // Noop
            }
            _ => return Ok(false),
        }
    }

    Ok(true)
}

// fn deserialize_instruction(index: usize, data: &[u8]) -> Result<Instruction, SanitizeError> {
//     const IS_SIGNER_BIT: usize = 0;
//     const IS_WRITABLE_BIT: usize = 1;

//     let mut current = 0;
//     let num_instructions = read_u16(&mut current, data)?;
//     if index >= num_instructions as usize {
//         return Err(SanitizeError::IndexOutOfBounds);
//     }

//     // index into the instruction byte-offset table.
//     current += index * 2;
//     let start = read_u16(&mut current, data)?;

//     current = start as usize;
//     let num_accounts = read_u16(&mut current, data)?;
//     let mut accounts = Vec::with_capacity(num_accounts as usize);
//     for _ in 0..num_accounts {
//         let meta_byte = read_u8(&mut current, data)?;
//         let mut is_signer = false;
//         let mut is_writable = false;
//         if meta_byte & (1 << IS_SIGNER_BIT) != 0 {
//             is_signer = true;
//         }
//         if meta_byte & (1 << IS_WRITABLE_BIT) != 0 {
//             is_writable = true;
//         }
//         let pubkey = read_pubkey(&mut current, data)?;
//         accounts.push(AccountMeta {
//             pubkey,
//             is_signer,
//             is_writable,
//         });
//     }
//     let program_id = read_pubkey(&mut current, data)?;
//     let data_len = read_u16(&mut current, data)?;
//     let data = read_slice(&mut current, data, data_len as usize)?;
//     Ok(Instruction {
//         program_id,
//         accounts,
//         data,
//     })
// }
