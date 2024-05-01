use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

use crate::{instruction::RegisterArgs, loaders::*};

/// Register generates a new hash chain for a prospective miner. Its responsibilities include:
/// 1. Initialize a new proof account.
/// 2. Generate an initial hash from the signer's key.
///
/// Safety requirements:
/// - Register is a permissionless instruction and can be invoked by any singer.
/// - Can only succeed if the provided proof acount PDA is valid (associated with the signer).
/// - Can only succeed once per signer.
/// - The provided system program must be valid.
pub fn process_deregister<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_proof(proof_info, signer.key, true)?;
    load_program(system_program, system_program::id())?;

    // TODO Ensure proof.balance == 0
    // TODO Send lamports to signer
    // TODO Realloc data to 0
    // TODO Reassign back to system program

    Ok(())
}
