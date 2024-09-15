use coal_api::state::{Proof, ProofV2};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult,
    program_error::ProgramError
};

use crate::utils::Discriminator;

use stake_coal::*;
use stake_wood::*;
mod stake_coal;
mod stake_wood;

pub fn process_stake<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let config_info = &accounts[1];

    if config_info.data.borrow()[0].eq(&(Proof::discriminator() as u8)) {
        return process_stake_coal(accounts, data)
    }

    if config_info.data.borrow()[0].eq(&(ProofV2::discriminator() as u8)) {
        return process_stake_wood(accounts, data)
    }

    return Err(ProgramError::InvalidAccountData);    
}