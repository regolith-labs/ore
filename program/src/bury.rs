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
    let [signer_info, sender_info, board_info, mint_info, treasury_info, treasury_ore_info, token_program, ore_program] =
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
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_ore_info.as_associated_token_account(treasury_info.key, &MINT_ADDRESS)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Transfer ORE from sender to treasury.
    let amount = sender.amount().min(amount);
    transfer(
        signer_info,
        sender_info,
        treasury_ore_info,
        token_program,
        amount,
    )?;

    // Share some ORE with stakers.
    let mut shared_amount = 0;
    if treasury.total_staked > 0 {
        shared_amount = amount / 10; // Share 10% of buyback ORE with stakers
        treasury.stake_rewards_factor +=
            Numeric::from_fraction(shared_amount, treasury.total_staked);
    }
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
