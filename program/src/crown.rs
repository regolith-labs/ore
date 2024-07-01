use ore_api::{
    loaders::*,
    state::{Config, Proof},
};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::utils::AccountDeserialize;

/// Crown flags an account as the top staker if their balance is greater than the last known top staker.
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

    // If top staker is the defualt null balance, skip this.
    if config.top_staker.ne(&Pubkey::new_from_array([0; 32])) {
        // Load current top staker
        load_any_proof(proof_info, false)?;
        if proof_info.key.ne(&config.top_staker) {
            return Ok(());
        }

        // Compare balances
        let proof_data = proof_info.data.borrow();
        let proof = Proof::try_from_bytes(&proof_data)?;
        if proof_new.balance.lt(&proof.balance) {
            return Ok(());
        }
    }

    // Crown the new top staker
    config.max_stake = proof_new.balance;
    config.top_staker = *proof_new_info.key;

    Ok(())
}
