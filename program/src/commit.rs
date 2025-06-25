use ore_api::prelude::*;
use steel::*;

/// Commit to a block.
pub fn process_commit(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Commit::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);
    let executor = Pubkey::new_from_array(args.executor);
    let fee = u64::from_le_bytes(args.fee);

    // Load accounts.
    let clock = Clock::get()?;
    let [signer_info, block_info, commitment_info, market_info, miner_info, mint_info, permit_info, sender_info, system_program, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| clock.slot < b.start_slot)?;
    commitment_info
        .has_address(&commitment_pda(block.id).0)?
        .as_token_account()?
        .assert(|t| t.mint() == *mint_info.key)?
        .assert(|t| t.owner() == *block_info.key)?;
    // commitment_info.as_associated_token_account(block_info.key, mint_info.key)?;
    let market = market_info
        .as_account::<Market>(&ore_api::ID)?
        .assert(|m| m.id == block.id)?;
    mint_info.has_address(&market.base.mint)?.as_mint()?;
    let sender = sender_info
        .is_writable()?
        .as_associated_token_account(signer_info.key, &mint_info.key)?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;

    // Normalize amount.
    let amount = sender.amount().min(amount);

    // Load miner account.
    let miner = if miner_info.data_is_empty() {
        create_program_account::<Miner>(
            miner_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[MINER, &signer_info.key.to_bytes()],
        )?;
        let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
        miner.authority = *signer_info.key;
        miner.block_id = 0;
        miner.hash = [0; 32];
        miner.total_committed = 0;
        miner.total_deployed = 0;
        miner.total_rewards = 0;
        miner
    } else {
        miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut(|m| m.authority == *signer_info.key)?
    };

    // Load permit account.
    let permit = if permit_info.data_is_empty() {
        create_program_account::<Permit>(
            permit_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[PERMIT, &signer_info.key.to_bytes(), &block.id.to_le_bytes()],
        )?;
        let permit = permit_info.as_account_mut::<Permit>(&ore_api::ID)?;
        permit.authority = *signer_info.key;
        permit.block_id = block.id;
        permit.commitment = 0;
        permit.executor = Pubkey::default();
        permit.fee = 0;
        permit.seed = [0; 32];
        permit
    } else {
        permit_info
            .as_account_mut::<Permit>(&ore_api::ID)?
            .assert_mut(|p| p.authority == *signer_info.key)?
            .assert_mut(|p| p.block_id == block.id)?
    };

    // Update executor logic.
    permit.executor = executor;
    permit.fee = fee;
    permit.seed = args.seed;

    // Transfer hash tokens.
    transfer(
        signer_info,
        sender_info,
        commitment_info,
        token_program,
        amount,
    )?;

    // Update block.
    permit.commitment += amount;
    miner.total_committed += amount;
    block.total_committed += amount;

    // Emit event.
    CommitEvent {
        disc: OreEvent::Commit as u64,
        authority: *signer_info.key,
        block_id: block.id,
        amount,
        block_commitment: block.total_committed,
        permit_commitment: permit.commitment,
        ts: clock.unix_timestamp,
    }
    .log_return();

    Ok(())
}
