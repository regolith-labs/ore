use coal_api::state::{Config, WoodConfig};
use solana_program::{
    account_info::AccountInfo, 
    entrypoint::ProgramResult,
    program_error::ProgramError,
};

use crate::utils::Discriminator;

use chop_wood::*;
use mine_coal::*;
mod chop_wood;
mod mine_coal;

pub fn process_mine<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let config_info = &accounts[2];

    if config_info.data.borrow()[0].eq(&(Config::discriminator() as u8)) {
        return process_mine_coal(accounts, data)
    }

    if config_info.data.borrow()[0].eq(&(WoodConfig::discriminator() as u8)) {
        return process_chop_wood(accounts, data)
    }

    return Err(ProgramError::InvalidAccountData);    
}