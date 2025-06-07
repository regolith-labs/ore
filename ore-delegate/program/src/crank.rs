use ore_api::prelude::*;
use ore_delegate_api::prelude::*;
use steel::*;

/// Cranks a mining transaction.
pub fn process_crank(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Crank::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts.
    let [signer_info, block_info, delegate_info, escrow_info, market_info, miner_info, mint_info, system_program, token_program, slot_hashes_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info.as_account::<Block>(&ore_api::ID)?;
    let delegate = delegate_info
        .as_account_mut::<Delegate>(&ore_delegate_api::ID)?
        .assert_mut(|d| d.block_id == block.id)?;

    // TODO Convert amount

    // Crank.
    let ix = ore_api::sdk::mine(*signer_info.key, block.id, amount);
    invoke_signed(
        &ix,
        &[
            delegate_info.clone(),
            block_info.clone(),
            market_info.clone(),
            miner_info.clone(),
            mint_info.clone(),
            escrow_info.clone(),
            system_program.clone(),
            token_program.clone(),
            slot_hashes_sysvar.clone(),
        ],
        &ore_delegate_api::ID,
        &[
            DELEGATE,
            &delegate.authority.to_bytes(),
            &block.id.to_le_bytes(),
        ],
    )?;

    // Pay fee.
    escrow_info.send(delegate.fee, &signer_info);

    Ok(())
}
