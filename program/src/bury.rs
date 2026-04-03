use ore_api::prelude::*;
use solana_program::log::sol_log;
use spl_token::amount_to_ui_amount;
use steel::*;

/// Bury ORE and distribute yield to stakers.
pub fn process_bury(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Bury::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, sender_info, board_info, mint_info, treasury_info, treasury_ore_info, stake_treasury_info, stake_treasury_tokens_info, token_program, ore_program, ore_stake_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let sender = sender_info
        .is_writable()?
        .as_associated_token_account(&signer_info.key, &MINT_ADDRESS)?;
    board_info.as_account_mut::<Board>(&ore_api::ID)?;
    mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_ore_info.as_associated_token_account(treasury_info.key, &MINT_ADDRESS)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;
    ore_stake_program.is_program(&ore_stake_api::ID)?;

    // Transfer ORE from sender to treasury.
    let amount = sender.amount().min(amount);
    transfer(
        signer_info,
        sender_info,
        treasury_ore_info,
        token_program,
        amount,
    )?;

    // Share 10% of buyback ORE with stakers
    let shared_amount = amount / 10;
    invoke_signed(
        &ore_stake_api::sdk::distribute(*treasury_info.key, shared_amount),
        &[
            treasury_info.clone(),
            treasury_ore_info.clone(),
            mint_info.clone(),
            stake_treasury_info.clone(),
            stake_treasury_tokens_info.clone(),
            token_program.clone(),
        ],
        &ore_api::ID,
        &[TREASURY],
    )?;
    sol_log(&format!(
        "💰 Shared {} ORE",
        amount_to_ui_amount(shared_amount, TOKEN_DECIMALS)
    ));

    // Burn ORE.
    let burn_amount = amount - shared_amount;
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
            sol_amount: 0,
            new_circulating_supply: mint.supply(),
            ts: Clock::get()?.unix_timestamp,
        }
        .to_bytes(),
    )?;

    Ok(())
}
