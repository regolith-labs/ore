use ore_api::{consts::*, instruction::StakeArgs, loaders::*, state::Proof};
use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::utils::AccountDeserialize;

/// Stake deposits Ore into a miner's proof account to earn multiplier. Its responsibilies include:
/// 1. Transfer tokens from the miner to the treasury account.
/// 2. Increment the miner's claimable balance.
///
/// Safety requirements:
/// - Stake is a permissionless instruction and can be called by any user.
/// - Can only succeed if the amount is less than or equal to the miner's transferable tokens.
/// - The provided beneficiary, proof, sender, treasury token account, and token program must be valid.
pub fn process_stake<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = StakeArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer, proof_info, sender_info, treasury_tokens_info, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_proof(proof_info, signer.key, true)?;
    load_token_account(sender_info, Some(signer.key), &MINT_ADDRESS, true)?;
    load_token_account(
        treasury_tokens_info,
        Some(&TREASURY_ADDRESS),
        &MINT_ADDRESS,
        true,
    )?;
    load_program(token_program, spl_token::id())?;

    // Update proof balance
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.balance = proof.balance.saturating_add(amount);

    // Update deposit timestamp
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    proof.last_stake_at = clock.unix_timestamp;

    // Distribute tokens from signer to treasury
    solana_program::program::invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            sender_info.key,
            treasury_tokens_info.key,
            signer.key,
            &[signer.key],
            amount,
        )?,
        &[
            token_program.clone(),
            sender_info.clone(),
            treasury_tokens_info.clone(),
            signer.clone(),
        ],
    )?;

    Ok(())
}
