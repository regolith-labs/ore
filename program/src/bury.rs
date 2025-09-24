use meteora_pools_sdk::instructions::SwapInstructionArgs;
use ore_api::prelude::*;
use solana_program::log::sol_log;
use solana_program::native_token::lamports_to_sol;
use solana_program::pubkey;
use solana_program::pubkey::Pubkey;
use spl_token::amount_to_ui_amount;
use steel::*;

const TOKEN_A_MINT: Pubkey = pubkey!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");
const TOKEN_B_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const METEORA_PROGRAM: Pubkey = pubkey!("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UaB");

/// Swap vaulted SOL to ORE, and burn the ORE.
pub fn process_bury(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Bury::try_from_bytes(data)?;
    let min_amount_out = u64::from_le_bytes(args.min_amount_out);

    // Load accounts.
    let (ore_accounts, meteora_accounts) = accounts.split_at(9);
    let [signer_info, config_info, mint_info, treasury_info, treasury_ore_info, treasury_sol_info, system_program, token_program, meteora_program] =
        ore_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    config_info
        .as_account::<Config>(&ore_api::ID)?
        .assert(|c| c.admin == *signer_info.key)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    let treasury_ore =
        treasury_ore_info.as_associated_token_account(treasury_info.key, &TOKEN_A_MINT)?;
    treasury_sol_info.as_associated_token_account(treasury_info.key, &TOKEN_B_MINT)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    meteora_program.is_program(&METEORA_PROGRAM)?;

    // Load meteora accounts.
    let [pool, user_source_token, user_destination_token, a_vault, b_vault, a_token_vault, b_token_vault, a_vault_lp_mint, b_vault_lp_mint, a_vault_lp, b_vault_lp, protocol_token_fee, user_key, vault_program, token_program] =
        meteora_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Sync native token balance.
    sync_native(treasury_sol_info)?;

    // Record pre-swap balances.
    let treasury_sol =
        treasury_sol_info.as_associated_token_account(treasury_info.key, &TOKEN_B_MINT)?;
    let pre_swap_ore_balance = treasury_ore.amount();
    let pre_swap_sol_balance = treasury_sol.amount();
    assert!(pre_swap_sol_balance > 0);
    sol_log(
        &format!(
            "Swapping {} SOL into ORE",
            lamports_to_sol(pre_swap_sol_balance),
        )
        .as_str(),
    );

    // Execute swap from SOL to ORE in Meteora
    let swap_ix = meteora_pools_sdk::instructions::Swap {
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
        user: *user_key.key,
        vault_program: *vault_program.key,
        token_program: *token_program.key,
    }
    .instruction(SwapInstructionArgs {
        in_amount: pre_swap_sol_balance,
        minimum_out_amount: min_amount_out,
    });
    invoke_signed(&swap_ix, meteora_accounts, &ore_api::ID, &[TREASURY])?;

    // Record post-swap balances.
    let treasury_ore =
        treasury_ore_info.as_associated_token_account(treasury_info.key, &TOKEN_A_MINT)?;
    let treasury_sol =
        treasury_sol_info.as_associated_token_account(treasury_info.key, &TOKEN_B_MINT)?;
    let post_swap_ore_balance = treasury_ore.amount();
    let post_swap_sol_balance = treasury_sol.amount();
    assert_eq!(post_swap_sol_balance, 0);

    // Burn ORE.
    let burn_amount = post_swap_ore_balance - pre_swap_ore_balance;
    burn_signed(
        treasury_ore_info,
        mint_info,
        treasury_info,
        token_program,
        burn_amount,
        &[TREASURY],
    )?;

    sol_log(
        &format!(
            "Buried {} ORE",
            amount_to_ui_amount(burn_amount, TOKEN_DECIMALS)
        )
        .as_str(),
    );

    Ok(())
}
