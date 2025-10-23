use ore_api::prelude::*;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Pays out the winners and block reward.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, board_info, config_info, fee_collector_info, mint_info, round_info, round_next_info, _top_miner_info, treasury_info, treasury_tokens_info, system_program, token_program, ore_program, slot_hashes_sysvar] =
        accounts
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

    // Sample slot hash.
    if config.is_new_rng_enabled == 0 {
        if let Ok(slot_hash) = get_slot_hash(board.end_slot, slot_hashes_sysvar) {
            round.slot_hash = slot_hash;
        } else {
            round.slot_hash = [u8::MAX; 32];
        }
    } else {
        if let Ok(slot_hash) = get_slot_hash_alt(board.end_slot, slot_hashes_sysvar) {
            round.slot_hash = slot_hash;
        } else {
            round.slot_hash = [u8::MAX; 32];
        }
    }

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

    // With 1 in 4 odds, split the +1 ORE reward.
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

pub fn get_slot_hash(
    slot: u64,
    slot_hashes_sysvar: &AccountInfo<'_>,
) -> Result<[u8; 32], ProgramError> {
    let slot_hashes =
        bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();
    let Some(slot_hash) = slot_hashes.get(&slot) else {
        // If reset is not called within ~2.5 minutes of the block ending,
        // then the slot hash will be unavailable and secure hashes cannot be generated.
        return Err(trace(
            "Slot hash unavailable",
            ProgramError::InvalidAccountData,
        ));
    };
    let slot_hash = slot_hash.to_bytes();
    Ok(slot_hash)
}

/// Alternative slot hash generation method.
///
/// The prior method gives a single leader full control over the slot hash, and thus the outcome of the round.
/// This method makes it significantly harder for a single leader to control the outcome by pigeonholing the outcome to 1 of 4 possible outcomes.
/// The available options are sampled by combing two hashes, selected 4 and 8 slots prior respectively.
/// Reasoning here being that no single validator is likely to be generate both hashes and the control hash, and thus the outcome of the round.
/// This rng solution does not remove *all* possibility of manipulation, but does make it significantly harder, raises the bar for collusion, and limits the range of possible outcomes.
pub fn get_slot_hash_alt(
    slot: u64,
    slot_hashes_sysvar: &AccountInfo<'_>,
) -> Result<[u8; 32], ProgramError> {
    let slot_hashes =
        bincode::deserialize::<SlotHashes>(slot_hashes_sysvar.data.borrow().as_ref()).unwrap();

    // Sample slot hashes 4 and 8 slots ago.
    // If reset is not called within ~2.5 minutes of the block ending,
    // then the slot hash will be unavailable and secure hashes cannot be generated.
    let slot_a = slot - 4;
    let slot_b = slot - 8;
    let slot_c = slot;
    let Some(slot_hash_a) = slot_hashes.get(&slot_a) else {
        return Err(trace(
            "Slot hash A unavailable",
            ProgramError::InvalidAccountData,
        ));
    };
    let Some(slot_hash_b) = slot_hashes.get(&slot_b) else {
        return Err(trace(
            "Slot hash B unavailable",
            ProgramError::InvalidAccountData,
        ));
    };
    let Some(slot_hash_c) = slot_hashes.get(&slot_c) else {
        return Err(trace(
            "Slot hash C unavailable",
            ProgramError::InvalidAccountData,
        ));
    };

    // Convert to bytes.
    let slot_hash_a = slot_hash_a.to_bytes();
    let slot_hash_b = slot_hash_b.to_bytes();
    let slot_hash_c = slot_hash_c.to_bytes();

    // Get control number.
    let r1 = u16::from_le_bytes(slot_hash_c[0..8].try_into().unwrap());
    let r2 = u16::from_le_bytes(slot_hash_c[8..16].try_into().unwrap());
    let r3 = u16::from_le_bytes(slot_hash_c[16..24].try_into().unwrap());
    let r4 = u16::from_le_bytes(slot_hash_c[24..32].try_into().unwrap());
    let c = (r1 ^ r2 ^ r3 ^ r4) % 2 == 0;

    // Select digest to use and generate final hash.
    let digest = if c { &slot_hash_a } else { &slot_hash_b };
    let slot_hash = solana_program::keccak::hash(digest.as_ref()).to_bytes();

    // Hash the final sample.
    Ok(slot_hash)
}
