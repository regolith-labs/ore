use ore_api::prelude::*;
use solana_program::log::sol_log;
use solana_program::native_token::lamports_to_sol;
use solana_program::pubkey;
use spl_token::amount_to_ui_amount;
use steel::*;

const JUPITER_PROGRAM_ID: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");

/// Swap vaulted SOL to ORE, and burn the ORE.
pub fn process_bury(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Load accounts.
    let (ore_accounts, swap_accounts) = accounts.split_at(9);
    let [signer_info, board_info, config_info, mint_info, treasury_info, treasury_ore_info, treasury_sol_info, token_program, ore_program] =
        ore_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    board_info.as_account_mut::<Board>(&ore_api::ID)?;
    config_info
        .as_account::<Config>(&ore_api::ID)?
        .assert(|c| c.bury_authority == *signer_info.key)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    let treasury_ore =
        treasury_ore_info.as_associated_token_account(treasury_info.key, &MINT_ADDRESS)?;
    treasury_sol_info.as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Sync native token balance.
    sync_native(treasury_sol_info)?;

    // Record pre-swap balances.
    let treasury_sol =
        treasury_sol_info.as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    let pre_swap_ore_balance = treasury_ore.amount();
    let pre_swap_sol_balance = treasury_sol.amount();
    assert!(pre_swap_sol_balance > 0);

    let accounts: Vec<AccountMeta> = swap_accounts
        .iter()
        .map(|acc| {
            let is_signer = acc.key == treasury_info.key;
            AccountMeta {
                pubkey: *acc.key,
                is_signer,
                is_writable: acc.is_writable,
            }
        })
        .collect();

    let accounts_infos: Vec<AccountInfo> = swap_accounts
        .iter()
        .map(|acc| AccountInfo { ..acc.clone() })
        .collect();

    invoke_signed(
        &Instruction {
            program_id: JUPITER_PROGRAM_ID,
            accounts,
            data: data.to_vec(),
        },
        &accounts_infos,
        &ore_api::ID,
        &[TREASURY],
    )?;

    // Record post-swap balances.
    let treasury_ore =
        treasury_ore_info.as_associated_token_account(treasury_info.key, &MINT_ADDRESS)?;
    let treasury_sol =
        treasury_sol_info.as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    let post_swap_ore_balance = treasury_ore.amount();
    let post_swap_sol_balance = treasury_sol.amount();
    let total_ore = post_swap_ore_balance - pre_swap_ore_balance;
    assert_eq!(post_swap_sol_balance, 0);
    sol_log(
        &format!(
            "📈 Swapped {} SOL into {} ORE",
            lamports_to_sol(pre_swap_sol_balance),
            amount_to_ui_amount(total_ore, TOKEN_DECIMALS),
        )
        .as_str(),
    );

    // Share some ORE with stakers.
    let mut shared_amount = 0;
    if treasury.total_staked > 0 {
        shared_amount = total_ore / 10; // Share 10% of buyback ORE with stakers
        treasury.stake_rewards_factor +=
            Numeric::from_fraction(shared_amount, treasury.total_staked);
    }

    sol_log(&format!(
        "💰 Shared {} ORE",
        amount_to_ui_amount(shared_amount, TOKEN_DECIMALS)
    ));

    // Burn ORE.
    let burn_amount = total_ore - shared_amount;
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
            "🔥 Buried {} ORE",
            amount_to_ui_amount(burn_amount, TOKEN_DECIMALS)
        )
        .as_str(),
    );

    // Emit event.
    let mint = mint_info.as_mint()?;
    program_log(
        &[board_info.clone(), ore_program.clone()],
        BuryEvent {
            disc: 1,
            ore_buried: burn_amount,
            ore_shared: shared_amount,
            sol_amount: pre_swap_sol_balance,
            new_circulating_supply: mint.supply(),
            ts: Clock::get()?.unix_timestamp,
        }
        .to_bytes(),
    )?;

    Ok(())
}
