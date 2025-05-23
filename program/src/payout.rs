use std::mem::size_of;

use ore_api::prelude::*;
use solana_program::{keccak::hashv, slot_hashes::SlotHash};
use steel::*;

pub fn process_payout(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, wager_info, recipient_info, treasury_info, treasury_tokens_info, system_program, token_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| b.ends_at < clock.slot)?
        .assert_mut(|b| b.payed_out == 0)?;
    let wager = wager_info.as_account::<Wager>(&ore_api::ID)?;
    recipient_info.as_associated_token_account(&wager.authority, &MINT_ADDRESS)?;
    treasury_info.has_address(&TREASURY_ADDRESS)?;
    treasury_tokens_info
        .has_address(&TREASURY_TOKENS_ADDRESS)?
        .is_writable()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Select the slothash from the slot at when the round ended.
    // The represents the server seed for the provably fair random number.
    let offset = clock.slot - block.ends_at;
    let size = size_of::<SlotHash>();
    let i = offset as usize * size;
    let slot_hash = &slot_hashes_sysvar.data.borrow()[i..i + size];
    block.noise = hashv(&[&block.noise, slot_hash]).to_bytes();

    // Calculate the random number.
    let x = u64::from_le_bytes(block.noise[0..8].try_into().unwrap());
    let y = u64::from_le_bytes(block.noise[8..16].try_into().unwrap());
    let z = u64::from_le_bytes(block.noise[16..24].try_into().unwrap());
    let w = u64::from_le_bytes(block.noise[24..32].try_into().unwrap());
    let roll = (x ^ y ^ z ^ w) % block.total_bets;

    // Assert that the wager account passed in is the winner.
    assert!(roll >= wager.cumulative_bets && roll < wager.cumulative_bets + wager.amount);

    // Mark the block as paid out.
    block.payed_out = 1;

    // Transfer the winnings to the recipient.
    transfer_signed(
        &treasury_info,
        &treasury_tokens_info,
        &recipient_info,
        &token_program,
        ONE_ORE / 2,
        &[TREASURY],
    )?;

    Ok(())
}
