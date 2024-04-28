use solana_program::{
    account_info::AccountInfo, clock::Clock, entrypoint::ProgramResult,
    program_error::ProgramError, pubkey::Pubkey, sysvar::Sysvar,
};

use crate::{
    error::OreError, instruction::StakeArgs, loaders::*, state::Proof, utils::AccountDeserialize,
    MINT_ADDRESS, TREASURY_ADDRESS,
};

pub fn process_stake<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = StakeArgs::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);

    // Load accounts
    let [signer_info, proof_info, sender_info, treasury_tokens_info, token_program] = accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer_info)?;
    load_proof(proof_info, signer_info.key, true)?;
    load_token_account(sender_info, Some(signer_info.key), &MINT_ADDRESS, true)?;
    load_token_account(
        treasury_tokens_info,
        Some(&TREASURY_ADDRESS),
        &MINT_ADDRESS,
        true,
    )?;
    load_program(token_program, spl_token::id())?;

    // Update proof balance
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.balance = proof
        .balance
        .checked_add(amount)
        .ok_or(OreError::StakeTooLarge)?;

    // Update deposit timestamp
    let clock = Clock::get().or(Err(ProgramError::InvalidAccountData))?;
    proof.last_deposit_slot = clock.slot;

    // Distribute tokens from signer to treasury
    solana_program::program::invoke(
        &spl_token::instruction::transfer(
            &spl_token::id(),
            sender_info.key,
            treasury_tokens_info.key,
            signer_info.key,
            &[signer_info.key],
            amount,
        )?,
        &[
            token_program.clone(),
            sender_info.clone(),
            treasury_tokens_info.clone(),
            signer_info.clone(),
        ],
    )?;

    Ok(())
}
