use entropy_api::state::Var;
use ore_api::prelude::*;
use solana_program::{keccak, log::sol_log, pubkey};
use steel::*;

pub const ORE_VAR_ADDRESS: Pubkey = pubkey!("BWCaDY96Xe4WkFq1M7UiCCRcChsJ3p51L5KrGzhxgm2E");

/// Pays out the winners and block reward.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let (ore_accounts, entropy_accounts) = accounts.split_at(14);
    sol_log(&format!("Ore accounts: {:?}", ore_accounts.len()).to_string());
    sol_log(&format!("Entropy accounts: {:?}", entropy_accounts.len()).to_string());
    let [signer_info, board_info, config_info, fee_collector_info, mint_info, round_info, round_next_info, _top_miner_info, treasury_info, treasury_tokens_info, system_program, token_program, ore_program, slot_hashes_sysvar] =
        ore_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| clock.slot >= b.end_slot + INTERMISSION_SLOTS)?;
    let config = config_info.as_account::<Config>(&ore_api::ID)?;
    fee_collector_info
        .is_writable()?
        .has_address(&config.fee_collector)?;
    let round = round_info
        .as_account_mut::<Round>(&ore_api::ID)?
        .assert_mut(|r| r.id == board.round_id)?;
    round_next_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[ROUND, &(board.round_id + 1).to_le_bytes()], &ore_api::ID)?;
    let mint = mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info.as_associated_token_account(&treasury_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    ore_program.is_program(&ore_api::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Open next round account.
    create_program_account::<Round>(
        round_next_info,
        ore_program,
        signer_info,
        &ore_api::ID,
        &[ROUND, &(board.round_id + 1).to_le_bytes()],
    )?;
    let round_next = round_next_info.as_account_mut::<Round>(&ore_api::ID)?;
    round_next.id = board.round_id + 1;
    round_next.deployed = [0; 25];
    round_next.slot_hash = [0; 32];
    round_next.count = [0; 25];
    round_next.expires_at = u64::MAX; // Set to max, to indicate round is waiting for first deploy to begin.
    round_next.rent_payer = *signer_info.key;
    round_next.motherlode = 0;
    round_next.top_miner = Pubkey::default();
    round_next.top_miner_reward = 0;
    round_next.total_deployed = 0;
    round_next.total_vaulted = 0;
    round_next.total_winnings = 0;

    // Sample random variable
    let [var_info, entropy_program] = entropy_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let var = var_info
        .has_address(&ORE_VAR_ADDRESS)? // TODO Verify address matches whats in the config.
        .as_account::<Var>(&entropy_api::ID)?
        .assert(|v| v.authority == *board_info.key)?
        .assert(|v| v.slot_hash != [0; 32])?
        .assert(|v| v.seed != [0; 32])?
        .assert(|v| v.value != [0; 32])?;
    entropy_program.is_program(&entropy_api::ID)?;

    // Print the seed and slot hash.
    let seed = keccak::Hash::new_from_array(var.seed);
    let slot_hash = keccak::Hash::new_from_array(var.slot_hash);
    sol_log(&format!("var slothash: {:?}", slot_hash).to_string());
    sol_log(&format!("var seed: {:?}", seed).to_string());

    // Read the finalized value from the var.
    let value = keccak::Hash::new_from_array(var.value);
    sol_log(&format!("var value: {:?}", value).to_string());
    round.slot_hash = var.value;

    // Exit early if no slot hash was found.
    let Some(r) = round.rng() else {
        // Slot hash could not be found, refund all SOL.
        round.total_vaulted = 0;
        round.total_winnings = 0;
        round.total_deployed = 0;

        // Emit event.
        program_log(
            &[board_info.clone(), ore_program.clone()],
            ResetEvent {
                disc: 0,
                round_id: round.id,
                start_slot: board.start_slot,
                end_slot: board.end_slot,
                winning_square: u64::MAX,
                top_miner: Pubkey::default(),
                num_winners: 0,
                motherlode: 0,
                total_deployed: round.total_deployed,
                total_vaulted: round.total_vaulted,
                total_winnings: round.total_winnings,
                total_minted: 0,
                ts: clock.unix_timestamp,
            }
            .to_bytes(),
        )?;

        // Update board for next round.
        board.round_id += 1;
        board.start_slot = clock.slot + 1;
        board.end_slot = u64::MAX;
        return Ok(());
    };

    // Caculate admin fees.
    let total_admin_fee = round.total_deployed / 100;

    // Get the winning square.
    let winning_square = round.winning_square(r);

    // If no one deployed on the winning square, vault all deployed.
    if round.deployed[winning_square] == 0 {
        // Vault all deployed.
        round.total_vaulted = round.total_deployed - total_admin_fee;
        treasury.balance += round.total_deployed - total_admin_fee;

        // Emit event.
        program_log(
            &[board_info.clone(), ore_program.clone()],
            ResetEvent {
                disc: 0,
                round_id: round.id,
                start_slot: board.start_slot,
                end_slot: board.end_slot,
                winning_square: winning_square as u64,
                top_miner: Pubkey::default(),
                num_winners: 0,
                motherlode: 0,
                total_deployed: round.total_deployed,
                total_vaulted: round.total_vaulted,
                total_winnings: round.total_winnings,
                total_minted: 0,
                ts: clock.unix_timestamp,
            }
            .to_bytes(),
        )?;

        // Update board for next round.
        board.round_id += 1;
        board.start_slot = clock.slot + 1;
        board.end_slot = u64::MAX;

        // Do SOL transfers.
        round_info.send(total_admin_fee, &fee_collector_info);
        round_info.send(round.total_deployed - total_admin_fee, &treasury_info);
        return Ok(());
    }

    // Get winnings amount (total deployed on all non-winning squares, minus admin fee).
    let winnings = round.calculate_total_winnings(winning_square);
    let winnings_admin_fee = winnings / 100; // 1% admin fee.
    let winnings = winnings - winnings_admin_fee;

    // Subtract vault amount from winnings.
    let vault_amount = winnings / 10; // 10% of winnings.
    let winnings = winnings - vault_amount;
    round.total_winnings = winnings;
    round.total_vaulted = vault_amount;
    treasury.balance += vault_amount;

    // Sanity check.
    assert!(
        round.total_deployed
            >= round.total_vaulted
                + round.total_winnings
                + round.deployed[winning_square]
                + winnings_admin_fee
    );

    // Mint +1 ORE for the winning miner(s).
    let mint_amount = MAX_SUPPLY.saturating_sub(mint.supply()).min(ONE_ORE);
    round.top_miner_reward = mint_amount;
    mint_to_signed(
        mint_info,
        treasury_tokens_info,
        treasury_info,
        token_program,
        mint_amount,
        &[TREASURY],
    )?;

    // With 1 in 2 odds, split the +1 ORE reward.
    if round.is_split_reward(r) {
        round.top_miner = SPLIT_ADDRESS;
    }

    // Payout the motherlode if it was activated.
    if round.did_hit_motherlode(r) {
        round.motherlode = treasury.motherlode;
        treasury.motherlode = 0;
    }

    // Mint +0.2 ORE to the motherlode rewards pool.
    let mint = mint_info.as_mint()?;
    let motherlode_mint_amount = MAX_SUPPLY.saturating_sub(mint.supply()).min(ONE_ORE / 5);
    if motherlode_mint_amount > 0 {
        mint_to_signed(
            mint_info,
            treasury_tokens_info,
            treasury_info,
            token_program,
            motherlode_mint_amount,
            &[TREASURY],
        )?;
        treasury.motherlode += motherlode_mint_amount;
    }

    // Validate top miner.
    // TODO Safety checks here (if no one won).
    // let mut top_miner_address = Pubkey::default();
    // let top_miner_sample = round.top_miner_sample(r, winning_square);
    // let top_miner = top_miner_info
    //     .as_account::<Miner>(&ore_api::ID)?
    //     .assert(|m| m.round_id == round.id)?
    //     .assert(|m| {
    //         m.cumulative[winning_square] >= top_miner_sample
    //             && top_miner_sample < m.cumulative[winning_square] + m.deployed[winning_square]
    //     })?;

    // Emit event.
    program_log(
        &[board_info.clone(), ore_program.clone()],
        ResetEvent {
            disc: 0,
            round_id: round.id,
            start_slot: board.start_slot,
            end_slot: board.end_slot,
            winning_square: winning_square as u64,
            top_miner: round.top_miner,
            motherlode: round.motherlode,
            num_winners: round.count[winning_square],
            total_deployed: round.total_deployed,
            total_vaulted: round.total_vaulted,
            total_winnings: round.total_winnings,
            total_minted: mint_amount + motherlode_mint_amount,
            ts: clock.unix_timestamp,
        }
        .to_bytes(),
    )?;

    // Reset board.
    board.round_id += 1;
    board.start_slot = clock.slot + 1;
    board.end_slot = u64::MAX; // board.start_slot + 150;

    // Do SOL transfers.
    round_info.send(total_admin_fee, &fee_collector_info);
    round_info.send(vault_amount, &treasury_info);

    Ok(())
}
