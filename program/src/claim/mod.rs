use coal_api::state::{Proof, ProofV2};
use coal_utils::Discriminator;
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult
};

use claim_coal::*;
use claim_wood::*;
mod claim_coal;
mod claim_wood;

/// Claim distributes claimable ORE from the treasury to a miner.
pub fn process_claim<'a, 'info>(accounts: &'a [AccountInfo<'info>], data: &[u8]) -> ProgramResult {
    let proof_info = &accounts[2];

    if proof_info.data.borrow()[0].eq(&(Proof::discriminator() as u8)) {
        return process_claim_coal(accounts, data)
    }

    if proof_info.data.borrow()[0].eq(&(ProofV2::discriminator() as u8)) {
        return process_claim_wood(accounts, data)
    }

    return Err(solana_program::program_error::ProgramError::InvalidAccountData);
}