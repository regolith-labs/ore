use ore_api::prelude::*;
use solana_program::keccak::hashv;
use steel::*;

/// Deploy capital to mine the current block.
pub fn process_deploy(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Deploy::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);
    let seed = args.seed;

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, block_commits_info, commit_info, sender_info, system_program, token_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| b.ends_at > clock.slot)?
        .assert_mut(|b| b.paid == 0)?;
    block_commits_info
        .is_writable()?
        .as_associated_token_account(block_info.key, &block.mint)?;
    commit_info.is_writable()?.is_empty()?.has_seeds(
        &[COMMIT, &block.current_round.to_le_bytes(), &seed],
        &ore_api::ID,
    )?;
    sender_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &block.mint)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Create commit account.
    create_program_account::<Commit>(
        &commit_info,
        &system_program,
        &signer_info,
        &ore_api::ID,
        &[COMMIT, &block.current_round.to_le_bytes(), &seed],
    )?;
    let commit = commit_info.as_account_mut::<Commit>(&ore_api::ID)?;
    commit.amount = amount;
    commit.authority = *signer_info.key;
    commit.cumulative_sum = block.cumulative_sum;
    commit.round = block.current_round;
    commit.seed = seed;
    commit.timestamp = clock.unix_timestamp as u64;

    // Update block stats.
    block.cumulative_sum += amount;
    block.total_commits += 1;

    // Hash client seed into block noise for provably fair randomness.
    block.noise = hashv(&[&block.noise, &seed]).to_bytes();

    // Transfer commits.
    transfer(
        &signer_info,
        &sender_info,
        &block_commits_info,
        &token_program,
        amount,
    )?;

    // Emit an event.
    DeployEvent {
        authority: *signer_info.key,
        amount,
        ts: clock.unix_timestamp as u64,
    }
    .log();

    Ok(())
}
