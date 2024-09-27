use ore_api::{consts::*, instruction::Stake, loaders::*, state::Proof};
use steel::*;

/// Stake deposits ORE into a proof account to earn multiplier.
pub fn process_stake(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Stake::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, proof_info, sender_info, treasury_tokens_info, token_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let proof = proof_info
        .to_account_mut::<Proof>(&ore_api::ID)?
        .check_mut(|p| p.authority == *signer_info.key)?;
    sender_info
        .is_writable()?
        .to_token_account()?
        .check(|t| t.owner == *signer_info.key)?
        .check(|t| t.mint == MINT_ADDRESS)?;
    treasury_tokens_info.is_writable()?.is_treasury_tokens()?;
    token_program.is_program(&spl_token::ID)?;

    // Update the proof balance.
    proof.balance = proof.balance.checked_add(amount).unwrap();

    // Update deposit timestamp.
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    proof.last_stake_at = clock.unix_timestamp;

    // Transfer tokens from signer to treasury.
    transfer(
        signer_info,
        sender_info,
        treasury_tokens_info,
        token_program,
        amount,
    )?;

    Ok(())
}
