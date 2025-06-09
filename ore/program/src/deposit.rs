use ore_api::prelude::*;
use solana_nostd_keccak::hash;
use solana_program::slot_hashes::SlotHashes;
use steel::*;

/// Deposits collateral.
pub fn process_deposit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Deposit::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, collateral_info, market_info, miner_info, mint_ore_info, sender_info, stake_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot < b.start_slot)?;
    collateral_info
        .is_writable()?
        .as_associated_token_account(block_info.key, mint_ore_info.key)?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.id == block.id)?;
    mint_ore_info.has_address(&MINT_ADDRESS)?.as_mint()?;
    sender_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_ore_info.key)?
        .assert(|t| t.amount() >= amount)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Load stake account.
    let stake = if stake_info.data_is_empty() {
        create_program_account::<Stake>(
            stake_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[STAKE, &signer_info.key.to_bytes(), &block.id.to_le_bytes()],
        )?;
        let stake = stake_info.as_account_mut::<Stake>(&ore_api::ID)?;
        stake.authority = *signer_info.key;
        stake.block_id = block.id;
        stake.capacity = 0;
        stake.utilization = 0;
        stake
    } else {
        stake_info
            .as_account_mut::<Stake>(&ore_api::ID)?
            .assert_mut(|p| p.authority == *signer_info.key)?
            .assert_mut(|p| p.block_id == block.id)?
    };

    // Update stake state.
    stake.capacity += amount;

    // Transfer collateral.
    transfer(
        sender_info,
        signer_info,
        collateral_info,
        token_program,
        amount,
    )?;

    Ok(())
}
