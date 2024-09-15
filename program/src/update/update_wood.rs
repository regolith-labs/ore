use coal_api::{consts::WOOD_MINT_ADDRESS, loaders::*, state::ProofV2};
use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
};

use crate::utils::AccountDeserialize;

pub fn process_update_wood<'a, 'info>(
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts.
    let [signer, miner_info, proof_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_any(miner_info, false)?;
    load_proof_v2(proof_info, signer.key, &WOOD_MINT_ADDRESS, true)?;

    // Update the proof's miner authority.
    let mut proof_data = proof_info.data.borrow_mut();
    let proof = ProofV2::try_from_bytes_mut(&mut proof_data)?;
    proof.miner = *miner_info.key;

    Ok(())
}
