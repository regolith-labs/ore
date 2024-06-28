use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::UpdateArgs, loaders::*, state::Proof, utils::AccountDeserialize};

/// Update updates a proof account.
pub fn process_update<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = UpdateArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_proof(proof_info, signer.key, true)?;

    // Update the proof
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = Proof::try_from_bytes_mut(&mut proof_data)?;
    proof.miner = args.new_miner;

    Ok(())
}
