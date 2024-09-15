use coal_api::state::{Proof, ProofV2};
use coal_utils::Discriminator;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult,
};

use close_coal::*;
use close_wood::*;
mod close_coal;
mod close_wood;

/// Close closes a proof account and returns the rent to the owner.
pub fn process_close<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let proof_info = &accounts[1];

    if proof_info.data.borrow()[0].eq(&(Proof::discriminator() as u8)) {
        return process_close_coal(accounts, data)
    }

    if proof_info.data.borrow()[0].eq(&(ProofV2::discriminator() as u8)) {
        return process_close_wood(accounts, data)
    }

    return Err(solana_program::program_error::ProgramError::InvalidAccountData);
}
