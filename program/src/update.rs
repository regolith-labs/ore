use coal_api::{loaders::*, state::{CoalProof, WoodProof}};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::utils::{AccountDeserialize, Discriminator};

pub fn process_update<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let config_info = &accounts[9];

    if config_info.data.borrow()[0].eq(&(CoalProof::discriminator() as u8)) {
        return process_update_coal(accounts, data)
    }

    if config_info.data.borrow()[0].eq(&(WoodProof::discriminator() as u8)) {
        return process_update_wood(accounts, data)
    }

    return Err(ProgramError::InvalidAccountData);    
}

/// Update changes the miner authority on a proof account.
pub fn process_update_coal<'a, 'info>(
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts.
    let [signer, miner_info, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any(miner_info, false)?;
    load_coal_proof(proof_info, signer.key, true)?;

    // Update the proof's miner authority.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = CoalProof::try_from_bytes_mut(&mut proof_data)?;
    proof.miner = *miner_info.key;

    Ok(())
}

pub fn process_update_wood<'a, 'info>(
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts.
    let [signer, miner_info, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any(miner_info, false)?;
    load_wood_proof(proof_info, signer.key, true)?;

    // Update the proof's miner authority.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = WoodProof::try_from_bytes_mut(&mut proof_data)?;
    proof.miner = *miner_info.key;

    Ok(())
}
