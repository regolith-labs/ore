use ore_api::prelude::*;
use solana_program::{
    log::sol_log,
    native_token::{lamports_to_sol, LAMPORTS_PER_SOL},
    pubkey,
    rent::Rent,
};
use steel::*;

/// Percentage of treasury SOL to send to the liq manager (whole unit, denominator 100).
const LIQ_PCT: u64 = 1;

/// The liq manager address.
const LIQ_MANAGER: Pubkey = pubkey!("Ag3AkRaEbqu3yEVibhEQsgEAxoLrC2MyEcxSEXRxfCuu");

/// Send SOL from the treasury to the WSOL account.
pub fn process_wrap(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Wrap::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, board_info, _config_info, manager_info, treasury_info, treasury_sol_info, system_program, ore_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&BURY_AUTHORITY)?;
    board_info.has_address(&BOARD_ADDRESS)?;
    manager_info.is_writable()?.has_address(&LIQ_MANAGER)?;
    treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_sol_info
        .is_writable()?
        .as_associated_token_account(treasury_info.key, &SOL_MINT)?;
    system_program.is_program(&system_program::ID)?;
    ore_program.is_program(&ore_api::ID)?;

    // Get amount
    let balance = treasury_info.lamports() - Rent::get()?.minimum_balance(Treasury::SIZE);
    let amount = (LAMPORTS_PER_SOL * 100).min(balance).min(amount);

    // Transfer liq percentage to the liq manager.
    let liq_amount = amount * LIQ_PCT / 100;
    if liq_amount > 0 {
        treasury_info.send(liq_amount, manager_info);
        sol_log(
            &format!(
                "Sent {} SOL to liq manager",
                lamports_to_sol(liq_amount)
            )
            .as_str(),
        );
    }

    // Send remaining SOL to the WSOL account for buyback.
    let wrap_amount = amount - liq_amount;
    treasury_info.send(wrap_amount, treasury_sol_info);

    // Check min balance.
    let min_balance = Rent::get()?.minimum_balance(Treasury::SIZE);
    assert!(
        treasury_info.lamports() >= min_balance,
        "Insufficient SOL balance"
    );

    // Emit liq event.
    let ts = Clock::get()?.unix_timestamp;
    program_log(
        &[board_info.clone(), ore_program.clone()],
        LiqEvent {
            disc: 3,
            sol_amount: liq_amount,
            recipient: *manager_info.key,
            ts,
        }
        .to_bytes(),
    )?;

    Ok(())
}
