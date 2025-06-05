use ore_api::prelude::*;
use steel::*;

/// Free up capacity.
pub fn process_free(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, miner_info, receipt_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.authority == *signer_info.key)?;
    let receipt = receipt_info
        .as_account_mut::<Receipt>(&ore_api::ID)?
        .assert_mut(|r| r.authority == *signer_info.key)?;

    // Asset that block has ended.
    let start_slot = 1500 * receipt.block_id;
    let end_slot = start_slot + 1500;
    assert!(clock.slot >= end_slot, "Block has not yet closed.");

    // Free up miner capacity.
    miner.deployed -= receipt.amount;

    // Close the receipt.
    receipt_info.close(signer_info)?;

    Ok(())
}
