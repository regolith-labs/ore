use std::mem::size_of;

use ore_api::prelude::*;
use solana_program::{keccak::hashv, slot_hashes::SlotHash};
use steel::*;

pub fn process_bet(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Bet::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, wager_info, block_bets_info, sender_info, system_program, token_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| b.ends_at > clock.slot)?
        .assert_mut(|b| b.payed_out != 0)?;
    block_bets_info.as_associated_token_account(block_info.key, &block.mint)?;
    sender_info.as_associated_token_account(signer_info.key, &block.mint)?;
    wager_info.is_writable()?.is_empty()?.has_seeds(
        &[
            WAGER,
            &block.current_round.to_le_bytes(),
            &block.bet_count.to_le_bytes(),
        ],
        &ore_api::ID,
    )?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Create wager account.
    create_program_account::<Wager>(
        &wager_info,
        &system_program,
        &signer_info,
        &ore_api::ID,
        &[
            WAGER,
            &block.current_round.to_le_bytes(),
            &block.bet_count.to_le_bytes(),
        ],
    )?;
    let wager = wager_info.as_account_mut::<Wager>(&ore_api::ID)?;
    wager.amount = amount;
    wager.authority = *signer_info.key;
    wager.id = block.bet_count;
    wager.round = block.current_round;
    wager.timestamp = clock.unix_timestamp as u64;
    wager.cumulative_bets = block.total_bets;

    // Update block.
    block.total_bets += amount;
    block.bet_count += 1;

    // Hash client seed into block noise. Use a recent slot hash if no seed is provided.
    // This follows the scheme for provable randomness.
    let seed: &[u8] = if args.seed == [0; 32] {
        &slot_hashes_sysvar.data.borrow()[0..size_of::<SlotHash>()]
    } else {
        args.seed.as_slice()
    };
    block.noise = hashv(&[&block.noise, seed]).to_bytes();

    // Transfer wagers.
    transfer(
        &signer_info,
        &sender_info,
        &block_bets_info,
        &token_program,
        amount,
    )?;

    Ok(())
}
