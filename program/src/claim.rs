use ore_api::prelude::*;
use steel::*;

/// Claim distributes claimable ORE from the treasury to a miner.
pub fn process_claim(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Claim::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, beneficiary_info, proof_info, treasury_info, treasury_tokens_info, vesting_info, token_program, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    beneficiary_info
        .is_writable()?
        .as_token_account()?
        .assert(|t| t.mint() == MINT_ADDRESS)?;
    let proof = proof_info
        .as_account_mut::<Proof>(&ore_api::ID)?
        .assert_mut_err(
            |p| p.authority == *signer_info.key,
            ProgramError::MissingRequiredSignature,
        )?;
    treasury_info.is_treasury()?;
    treasury_tokens_info.is_writable()?.is_treasury_tokens()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Create vesting account if it doesn't exist.
    let vesting = if vesting_info.data_is_empty() {
        // Verify seeds
        vesting_info
            .is_empty()?
            .is_writable()?
            .has_seeds(&[VESTING, proof_info.key.as_ref()], &ore_api::ID)?;

        // Initialize vesting.
        create_program_account::<Vesting>(
            vesting_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[VESTING, proof_info.key.as_ref()],
        )?;
        let vesting = vesting_info
            .as_account_mut::<Vesting>(&ore_api::ID)?
            .assert_mut(|v| v.proof == *proof_info.key)?;
        vesting.proof = *proof_info.key;
        vesting.window_claim_amount = 0;
        vesting.window_proof_balance = proof.balance;
        vesting.window_start_at = clock.unix_timestamp;
        vesting
    } else {
        // Load vesting account.
        vesting_info
            .as_account_mut::<Vesting>(&ore_api::ID)?
            .assert_mut(|v| v.proof == *proof_info.key)?
    };

    // Update vesting window.
    if clock.unix_timestamp > vesting.window_start_at + ONE_DAY {
        vesting.window_claim_amount = 0;
        vesting.window_proof_balance = proof.balance;
        vesting.window_start_at = clock.unix_timestamp;
    } else {
        vesting.window_proof_balance = vesting.window_proof_balance.max(proof.balance);
    };

    // Calculate claim amount.
    let max_claim_amount = vesting.window_proof_balance.checked_div(100).unwrap();
    let remaining_claim_amount = max_claim_amount
        .checked_sub(vesting.window_claim_amount)
        .unwrap();

    // Exempt boost proof from vesting.
    let boost_config_address = ore_boost_api::state::config_pda().0;
    let claim_amount = if proof.authority == boost_config_address {
        amount.min(proof.balance)
    } else {
        amount.min(remaining_claim_amount).min(proof.balance)
    };

    // Update miner balance.
    proof.balance = proof
        .balance
        .checked_sub(claim_amount)
        .ok_or(OreError::ClaimTooLarge)?;

    // Update last claim timestamp.
    proof.last_claim_at = clock.unix_timestamp;

    // Transfer tokens from treasury to beneficiary.
    transfer_signed(
        treasury_info,
        treasury_tokens_info,
        beneficiary_info,
        token_program,
        claim_amount,
        &[TREASURY],
    )?;

    Ok(())
}
