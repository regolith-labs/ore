use coal_api::state::{Proof, ProofV2};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::utils::Discriminator;

use update_coal::*;
use update_wood::*;
mod update_coal;
mod update_wood;

/// Update changes the miner authority on a proof account.
pub fn process_update<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let config_info = &accounts[9];

    if config_info.data.borrow()[0].eq(&(Proof::discriminator() as u8)) {
        return process_update_coal(accounts, data)
    }

    if config_info.data.borrow()[0].eq(&(ProofV2::discriminator() as u8)) {
        return process_update_wood(accounts, data)
    }

    return Err(ProgramError::InvalidAccountData);    
}
