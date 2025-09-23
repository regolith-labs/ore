use ore_api::prelude::*;
use solana_program::rent::Rent;
use steel::*;

/// Reimburses an executor keypair for their tx fee expenses.
pub fn process_reimburse(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Reimburse::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let miner = miner_info
        .as_account_mut::<Miner>(&ore_api::ID)?
        .assert_mut(|m| m.executor == *signer_info.key)?;
    system_program.is_program(&system_program::ID)?;

    // Check if miner can charge.// Check total amount.
    let account_size = 8 + std::mem::size_of::<Miner>();
    let min_rent = Rent::get()?.minimum_balance(account_size);
    let claimable_sol = miner.rewards_sol;
    let obligations = min_rent + claimable_sol;
    let new_lamports = miner_info.lamports().saturating_sub(amount);
    if new_lamports < obligations {
        return Err(trace(
            "Miner account has insufficient SOL",
            ProgramError::InsufficientFunds,
        ));
    }

    // Send reimbursement.
    miner_info.send(amount, signer_info);

    Ok(())
}
