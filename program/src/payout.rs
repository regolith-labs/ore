use ore_api::prelude::*;
use solana_program::keccak::hashv;
use steel::*;
use sysvar::slot_hashes::SlotHashes;

/// Pays out a block reward to the winning.
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
    treasury_info.has_address(&TREASURY_ADDRESS)?;
    treasury_tokens_info
        .has_address(&TREASURY_TOKENS_ADDRESS)?
        .is_writable()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Mark the block as paid out.
    block.payed_out = 1;

    // Skip payout if no bets were placed.
    if block.total_bets == 0 {
        return Ok(());
    }

    // Select the slothash from the slot at when the round ended.
    // The represents the server seed for a provably fair random number.
    let slot_hashes =
        bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
    let slot_hash = slot_hashes.get(&block.ends_at).unwrap();
    block.noise = hashv(&[&block.noise, slot_hash.as_ref()]).to_bytes();

    // Calculate the random number.
    let x = u64::from_le_bytes(block.noise[0..8].try_into().unwrap());
    let y = u64::from_le_bytes(block.noise[8..16].try_into().unwrap());
    let z = u64::from_le_bytes(block.noise[16..24].try_into().unwrap());
    let w = u64::from_le_bytes(block.noise[24..32].try_into().unwrap());
    let roll = (x ^ y ^ z ^ w) % block.total_bets;

    // Validate the wager account.
    let wager = wager_info
        .as_account_mut::<Wager>(&ore_api::ID)?
        .assert_mut(|w| roll >= w.cumulative_bets)?
        .assert_mut(|w| roll < w.cumulative_bets + w.amount)?;
    recipient_info.as_associated_token_account(&wager.authority, &MINT_ADDRESS)?;

    // Transfer the winnings to the recipient.
    transfer_signed(
        &treasury_info,
        &treasury_tokens_info,
        &recipient_info,
        &token_program,
        block.reward,
        &[TREASURY],
    )?;

    Ok(())
}
