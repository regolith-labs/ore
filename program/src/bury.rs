use meteora_pools_sdk::instructions::SwapInstructionArgs;
use ore_api::prelude::*;
use steel::*;

/// Redeem ORE for SOL backing.
pub fn process_redeem(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Bury::try_from_bytes(data)?;
    let min_amount_out = u64::from_le_bytes(args.min_amount_out);

    // Load accounts.
    let [ore_accounts, meteora_accounts] = accounts.split_at(6);
    let [signer_info, config_info, mint_info, treasury_info, system_program, token_program] =
        ore_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account::<Config>(&ore_api::ID)?
        .assert_mut(|c| c.admin == *signer_info.key)?;
    let mint = mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    let [pool, user_source_token, user_destination_token, a_vault, b_vault, a_token_vault, b_token_vault, a_vault_lp_mint, b_vault_lp_mint, a_vault_lp, b_vault_lp, protocol_token_fee, vault_program] =
        meteora_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Execute swap from SOL to ORE in Meteora
    let swap = meteora_pools_sdk::instructions::Swap {
        pool: *pool.key,
        user_source_token: *user_source_token.key,
        user_destination_token: *user_destination_token.key,
        a_vault: *a_vault.key,
        b_vault: *b_vault.key,
        a_token_vault: *a_token_vault.key,
        b_token_vault: *b_token_vault.key,
        a_vault_lp_mint: *a_vault_lp_mint.key,
        b_vault_lp_mint: *b_vault_lp_mint.key,
        a_vault_lp: *a_vault_lp.key,
        b_vault_lp: *b_vault_lp.key,
        protocol_token_fee: *protocol_token_fee.key,
        user: *user.key,
        vault_program: *vault_program.key,
        token_program: *token_program.key,
        b_token_vault: *b_token_vault.key,
        a_vault_lp_mint: *a_vault_lp_mint.key,
        b_vault_lp_mint: *b_vault_lp_mint.key,
        a_vault_lp: *a_vault_lp.key,
        b_vault_lp: *b_vault_lp.key,
        protocol_token_fee: *protocol_token_fee.key,
        user: *user.key,
        vault_program: *vault_program.key,
        token_program: *token_program.key,
    }
    .instruction_with_remaining_accounts(
        SwapInstructionArgs {
            in_amount: treasury.balance,
            minimum_out_amount: min_amount_out,
        },
        &meteora_accounts,
    );

    // Burn ORE.
    burn(sender_info, mint_info, signer_info, token_program, amount)?;

    // // Transfer SOL to recipient.
    // assert!(
    //     treasury.balance >= redemption_amount,
    //     "Redemption too large"
    // );
    // treasury_info.send(redemption_amount, signer_info);
    // treasury.balance -= redemption_amount;

    Ok(())
}
