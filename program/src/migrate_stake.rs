use solana_program::pubkey;
use steel::*;

const MIGRATE_AUTHORITY: Pubkey = pubkey!("HBUh9g46wk2X89CvaNN15UmsznP59rh6od1h8JwYAopk");

/// Migrates ORE from the old staking contract to the new one.
pub fn process_migrate_stake(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, payer_info, mint_info, old_stake_info, old_stake_tokens_info, stake_info, stake_tokens_info, old_treasury_info, new_treasury_info, new_treasury_tokens_info, system_program, token_program, associated_token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&MIGRATE_AUTHORITY)?;
    payer_info.is_signer()?;
    mint_info
        .has_address(&ore_api::prelude::MINT_ADDRESS)?
        .as_mint()?;
    let old_stake = old_stake_info.as_account_mut::<ore_api::prelude::Stake>(&ore_api::ID)?;
    old_stake_tokens_info.as_associated_token_account(old_stake_info.key, mint_info.key)?;
    stake_info.is_writable()?.is_empty()?;
    stake_tokens_info.is_writable()?.is_empty()?;
    let old_treasury =
        old_treasury_info.as_account_mut::<ore_api::prelude::Treasury>(&ore_api::ID)?;
    new_treasury_info.as_account_mut::<ore_stake_api::prelude::Treasury>(&ore_stake_api::ID)?;
    new_treasury_tokens_info
        .is_writable()?
        .as_associated_token_account(&new_treasury_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;

    // Update rewards
    old_stake.update_rewards(old_treasury);

    invoke_signed(
        &ore_stake_api::sdk::migrate_stake(*payer_info.key, old_stake.authority),
        &[
            payer_info.clone(),
            mint_info.clone(),
            old_stake_info.clone(),
            old_stake_tokens_info.clone(),
            stake_info.clone(),
            stake_tokens_info.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
        &ore_api::ID,
        &[ore_api::prelude::STAKE, &old_stake.authority.to_bytes()],
    )?;

    // Transfer deposits
    transfer_signed(
        old_stake_info,
        old_stake_tokens_info,
        stake_tokens_info,
        token_program,
        old_stake.balance,
        &[ore_api::prelude::STAKE, &old_stake.authority.to_bytes()],
    )?;

    // Transfer yield
    transfer_signed(
        old_treasury_info,
        old_stake_tokens_info,
        new_treasury_tokens_info,
        token_program,
        old_stake.rewards,
        &[ore_stake_api::prelude::TREASURY],
    )?;

    // Close old stake account
    old_stake_info.close(payer_info)?;
    old_stake_tokens_info.close(payer_info)?;

    // Refund compound fee reserve
    stake_info.collect(old_stake.compound_fee_reserve, payer_info)?;

    panic!("Just testing");

    Ok(())
}
