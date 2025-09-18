use ore_api::prelude::*;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Claims a block reward.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let (required_accounts, miner_accounts) = accounts.split_at(8);
    let [signer_info, board_info, mint_info, square_info, treasury_info, treasury_tokens_info, system_program, token_program, slot_hashes_sysvar] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let board = board_info
        .as_account_mut::<Board>(&ore_api::ID)?
        .assert_mut(|b| b.slot_hash == [0; 32])?
        .assert_mut(|b| clock.slot >= b.end_slot)?;
    let mint = mint_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    let square = square_info.as_account_mut::<Square>(&ore_api::ID)?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    treasury_tokens_info.as_associated_token_account(&treasury_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Sample slot hash.
    let (winning_square, square_prospects) =
        if let Ok(slot_hash) = get_slot_hash(board.end_slot, slot_hashes_sysvar) {
            board.slot_hash = slot_hash;
            let winning_square = get_winning_square(&slot_hash);
            let square_prospects = board.prospects[winning_square];
            (winning_square, square_prospects)
        } else {
            // Cannot get slot hash. No one wins.
            board.slot_hash = [u8::MAX; 32];
            (u64::MAX, 0)
        };

    // No one won. Vault all prospects.
    if square_prospects == 0 {
        board.total_vaulted = board.total_prospects;
        treasury.balance += board.total_prospects;
        board_info.send(board.total_prospects, &treasury_info);
        return Ok(());
    }

    // Get winnings amount (prospects on all non-winning squares).
    let mut winnings = 0;
    for (i, prospects) in board.prospects.iter().enumerate() {
        if i as u64 != winning_square {
            winnings += prospects;
        }
    }

    // Get vault amount (backing ORE floor price with SOL).
    let vault_amount = winnings / 10; // 10% of winnings.
    let winnings = winnings - vault_amount;
    board.total_vaulted = vault_amount;
    treasury.balance += vault_amount;

    // Record miner rewards.
    let mut top_miner = None;
    let mut top_miner_prospects = 0;
    let mut rewards_sol = [0; 16];
    let mut checksum = 0;
    for (i, miner_info) in miner_accounts.iter().enumerate() {
        let miner = miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.round_id == board.id)?;
        let miner_prospects = miner.prospects[winning_square];
        let rewards = miner_prospects + (winnings * miner_prospects / square_prospects); // Winners get their own prospect back plus their share of the winnings.
        checksum += miner_prospects;
        miner.rewards_sol += rewards;
        miner.lifetime_rewards_sol += rewards;
        rewards_sol[i] = rewards;

        // Check if miner was provided in correct order.
        if miner.authority != square.miners[winning_square][i] {
            return Err(ProgramError::InvalidAccountData);
        }

        // Find the top winner.
        if miner_prospects > top_miner_prospects {
            top_miner_prospects = miner_prospects;
            top_miner = Some(i);
        }
    }

    // Verify checksum.
    if checksum != square_prospects {
        // This can only happen if the caller didn't provide full set of winning miners.
        return Err(ProgramError::InvalidAccountData);
    }

    // Payout reward to top miner.
    if let Some(i) = top_miner {
        let miner = miner_accounts[i].as_account_mut::<Miner>(&ore_api::ID)?;
        let mint_amount = ONE_ORE.min(MAX_SUPPLY - mint.supply());
        if mint_amount > 0 {
            miner.rewards_ore += mint_amount;
            miner.lifetime_rewards_ore += mint_amount;
            board.top_miner = miner.authority;
            mint_to_signed(
                mint_info,
                treasury_tokens_info,
                treasury_info,
                token_program,
                mint_amount,
                &[TREASURY],
            )?;
        }
    }

    // Update board.
    board.total_winnings = winnings;

    // Do SOL transfers.
    board_info.send(vault_amount, &treasury_info);
    for (i, miner_info) in miner_accounts.iter().enumerate() {
        board_info.send(rewards_sol[i], &miner_info);
    }

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
        return Err(ProgramError::InvalidAccountData);
    };
    let slot_hash = slot_hash.to_bytes();
    Ok(slot_hash)
}

fn get_winning_square(slot_hash: &[u8]) -> usize {
    // Use slot hash to generate a random u64
    let r1 = u64::from_le_bytes(slot_hash[0..8].try_into().unwrap());
    let r2 = u64::from_le_bytes(slot_hash[8..16].try_into().unwrap());
    let r3 = u64::from_le_bytes(slot_hash[16..24].try_into().unwrap());
    let r4 = u64::from_le_bytes(slot_hash[24..32].try_into().unwrap());
    let r = r1 ^ r2 ^ r3 ^ r4;

    // Returns a value in the range [0, 24] inclusive
    (r % 25) as usize
}
