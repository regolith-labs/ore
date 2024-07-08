use ore_api::{
    error::OreError,
    loaders::*,
    state::{Config, Proof},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, system_program,
};

use crate::utils::AccountDeserialize;

/// Close closes a proof account and returns the rent to the owner. Its responsibilities include:
/// 1. Realloc proof account size to 0.
/// 2. Transfer lamports to the owner.
///
/// Safety requirements:
/// - Deregister is a permissionless instruction and can be invoked by any singer.
/// - Can only succeed if the provided proof acount PDA is valid (associated with the signer).
/// - The provided system program must be valid.
pub fn process_close<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, config_info, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_config(config_info, false)?;
    load_proof(proof_info, signer.key, true)?;
    load_program(system_program, system_program::id())?;

    // Validate the account is not the crowned top staker.
    let config_data = config_info.data.borrow();
    let config = Config::try_from_bytes(&config_data)?;
    if config.top_staker.eq(proof_info.key) {
        return Err(OreError::CannotClose.into());
    }

    // Validate balance is zero
    let proof_data = proof_info.data.borrow();
    let proof = Proof::try_from_bytes(&proof_data)?;
    if proof.balance.gt(&0) {
        return Err(ProgramError::InvalidAccountData);
    }
    drop(proof_data);

    // Realloc data to zero
    proof_info.realloc(0, true)?;

    // Send lamports to signer
    **signer.lamports.borrow_mut() += proof_info.lamports();
    **proof_info.lamports.borrow_mut() = 0;

    Ok(())
}
