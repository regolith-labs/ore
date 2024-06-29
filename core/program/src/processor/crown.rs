use ore_api::{
    state::{Config, Proof},
    utils::AccountDeserialize,
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::loaders::*;

/// Crown marks an account as the top staker if their balance is greater than the last known top staker.
pub fn process_crown<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, config_info, proof_info, proof_new_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_config(config_info, true)?;
    load_any_proof(proof_new_info, false)?;

    // Load config
    let mut config_data = config_info.data.borrow_mut();
    let config = Config::try_from_bytes_mut(&mut config_data)?;

    // Load proposed new top staker
    let proof_new_data = proof_new_info.data.borrow();
    let proof_new = Proof::try_from_bytes(&proof_new_data)?;

    // If top staker is not the default null address, then compare balances
    if config.top_staker.ne(&Pubkey::new_from_array([0; 32])) {
        // Load current top staker
        load_any_proof(proof_info, false)?;
        let proof_data = proof_info.data.borrow();
        let proof = Proof::try_from_bytes(&proof_data)?;

        // Require the provided proof account is the current top staker
        if config.top_staker.ne(&proof_info.key) {
            return Ok(());
        }

        // Compare balances
        if proof_new.balance.lt(&proof.balance) {
            return Ok(());
        }
    }

    // Crown the new top staker
    config.max_stake = proof_new.balance;
    config.top_staker = *proof_new_info.key;

    Ok(())
}
