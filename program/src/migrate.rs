use ore_api::prelude::*;
use solana_program::hash;
use steel::*;

/// Mine validates hashes and increments a miner's claimable balance.
pub fn process_migrate(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Migrate::try_from_bytes(data)?;

    // Load accounts.
    let clock = Clock::get()?;
    let t: i64 = clock.unix_timestamp;
    let [signer_info, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, mint_info, treasury_info, treasury_tokens_info, token_program, system_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&INITIALIZER_ADDRESS);
    let config = config_info
        .as_account_mut::<OldConfig>(&ore_api::ID)?
        .assert_mut_err(
            |c| t < c.last_reset_at + EPOCH_DURATION,
            OreError::NeedsReset.into(),
        )?;
    let bus_0 = bus_0_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 0)?;
    let bus_1 = bus_1_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 1)?;
    let bus_2 = bus_2_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 2)?;
    let bus_3 = bus_3_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 3)?;
    let bus_4 = bus_4_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 4)?;
    let bus_5 = bus_5_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 5)?;
    let bus_6 = bus_6_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 6)?;
    let bus_7 = bus_7_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 7)?;
    mint_info
        .is_writable()?
        .has_address(&MINT_ADDRESS)?
        .as_mint()?;
    treasury_info
        .has_address(&TREASURY_ADDRESS)?
        .is_writable()?;
    treasury_tokens_info
        .has_address(&TREASURY_TOKENS_ADDRESS)?
        .as_associated_token_account(&TREASURY_ADDRESS, &MINT_ADDRESS)?;
    token_program.is_program(&spl_token::ID)?;
    system_program.is_program(&system_program::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    let mut total_bus_balance = 0;
    total_bus_balance += bus_0.rewards;
    total_bus_balance += bus_1.rewards;
    total_bus_balance += bus_2.rewards;
    total_bus_balance += bus_3.rewards;
    total_bus_balance += bus_4.rewards;
    total_bus_balance += bus_5.rewards;
    total_bus_balance += bus_6.rewards;
    total_bus_balance += bus_7.rewards;

    // Reset bus balances
    bus_0.rewards = 0;
    bus_1.rewards = 0;
    bus_2.rewards = 0;
    bus_3.rewards = 0;
    bus_4.rewards = 0;
    bus_5.rewards = 0;
    bus_6.rewards = 0;
    bus_7.rewards = 0;

    // Delete bus accounts
    bus_0_info.close(signer_info)?;
    bus_1_info.close(signer_info)?;
    bus_2_info.close(signer_info)?;
    bus_3_info.close(signer_info)?;
    bus_4_info.close(signer_info)?;
    bus_5_info.close(signer_info)?;
    bus_6_info.close(signer_info)?;
    bus_7_info.close(signer_info)?;

    // Burn all tokens in the bus balances
    burn_signed(
        treasury_tokens_info,
        mint_info,
        treasury_info,
        token_program,
        total_bus_balance,
        &[TREASURY],
    )?;

    // let proof = proof_info
    //     .as_account_mut::<Proof>(&ore_api::ID)?
    //     .assert_mut_err(
    //         |p| p.miner == *signer_info.key,
    //         ProgramError::MissingRequiredSignature,
    //     )?;

    // Compute the hash.
    // let solution = hash::hashv(&[
    //     args.nonce.as_slice(),
    //     config.challenge.as_slice(),
    //     proof.authority.to_bytes().as_slice(),
    // ]);

    // // Update the best solution.
    // if solution.to_bytes() < config.best_hash {
    //     config.best_hash = solution.to_bytes();
    //     config.best_proof = *proof_info.key;
    // }

    Ok(())
}
