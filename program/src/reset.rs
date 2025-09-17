use ore_api::prelude::*;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Claims a block reward.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let (required_accounts, miner_accounts) = accounts.split_at(9);
    let [signer_info, board_info, mint_info, treasury_info, reserve_tokens_info, vault_info, system_program, token_program, slot_hashes_sysvar] =
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
    reserve_tokens_info
        .has_address(&BOOST_RESERVE_TOKEN)?
        .as_token_account()?
        .assert(|t| t.mint() == MINT_ADDRESS)?;
    vault_info.has_address(&vault_address())?;
    treasury_info.has_address(&TREASURY_ADDRESS)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Mint tokens to the boost reserve.
    mint_to_signed(
        mint_info,
        reserve_tokens_info,
        treasury_info,
        token_program,
        ONE_ORE / 3,
        &[TREASURY],
    )?;

    // Sample slot hashes.
    let (winning_square, square_commits) =
        if let Ok(slot_hash) = get_slot_hash(board.end_slot, slot_hashes_sysvar) {
            board.slot_hash = slot_hash;
            let winning_square = get_winning_square(&slot_hash);
            let square_commits = board.commits[winning_square as usize];
            (winning_square, square_commits)
        } else {
            // Cannot get slot hash. No one wins.
            board.slot_hash = [u8::MAX; 32];
            (u64::MAX, 0)
        };

    // No one won. Burn all rewards.
    if square_commits == 0 {
        board.total_burned = board.total_commits;
        burn_signed(
            vault_info,
            mint_info,
            board_info,
            token_program,
            board.total_commits,
            &[BOARD],
        )?;
        return Ok(());
    }

    // Get winnings amount (prospects on all non-winning squares).
    let mut winnings = 0;
    for (i, commits) in board.commits.iter().enumerate() {
        if i as u64 != winning_square {
            winnings += commits;
        }
    }

    // Get burn amount.
    let burn_amount = winnings / 10; // Burn 10% of non-winning prospects.
    board.total_burned = burn_amount;
    let winnings = winnings - burn_amount;
    burn_signed(
        vault_info,
        mint_info,
        board_info,
        token_program,
        burn_amount,
        &[BOARD],
    )?;

    // Mint 1 ORE to winners while there are emissions left.
    let mint_amount = ONE_ORE.min(MAX_SUPPLY - mint.supply());
    let winnings = winnings + mint_amount;
    if mint_amount > 0 {
        mint_to_signed(
            mint_info,
            vault_info,
            treasury_info,
            token_program,
            mint_amount,
            &[TREASURY],
        )?;
    }

    // Payout winnings to miners.
    let mut checksum = 0;
    for miner_info in miner_accounts {
        let miner = miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.round_id == board.id)?;
        let miner_commits = miner.commits[winning_square as usize];
        let rewards = (winnings * miner_commits / square_commits) + miner_commits; // Winners get their own prospect back plus their share of the winnings.
        miner.rewards += rewards;
        miner.total_rewards += rewards;
        checksum += miner_commits;
    }

    // Verify checksum.
    if checksum != square_commits {
        // This can only happen if the caller didn't provide full set of winning miners.
        return Err(ProgramError::InvalidAccountData);
    }

    // Update board.
    board.total_winnings = winnings;

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

fn get_winning_square(slot_hash: &[u8]) -> u64 {
    // Use slot hash to generate a random u64
    let r1 = u64::from_le_bytes(slot_hash[0..8].try_into().unwrap());
    let r2 = u64::from_le_bytes(slot_hash[8..16].try_into().unwrap());
    let r3 = u64::from_le_bytes(slot_hash[16..24].try_into().unwrap());
    let r4 = u64::from_le_bytes(slot_hash[24..32].try_into().unwrap());
    let r = r1 ^ r2 ^ r3 ^ r4;

    // Returns a value in the range [0, 24] inclusive
    r % 25
}
