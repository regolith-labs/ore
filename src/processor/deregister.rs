use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program::invoke_signed,
    program_error::ProgramError, pubkey::Pubkey, system_program,
};

use crate::{loaders::*, state::Proof, utils::AccountDeserialize, PROOF};

/// Deregister closes a proof account and returns the rent to the owner. Its responsibilities include:
/// 1. Realloc proof account size to 0.
/// 2. Transfer lamports to the owner.
/// 3. Reassign the account owner back to the system program.
///
/// Safety requirements:
/// - Deregister is a permissionless instruction and can be invoked by any singer.
/// - Can only succeed if the provided proof acount PDA is valid (associated with the signer).
/// - The provided system program must be valid.
pub fn process_deregister<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    _data: &[u8],
) -> ProgramResult {
    // Load accounts
    let [signer, proof_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_proof(proof_info, signer.key, true)?;
    load_program(system_program, system_program::id())?;

    // Validate balance is zero
    let proof_data = proof_info.data.borrow();
    let proof = Proof::try_from_bytes(&proof_data)?;
    if proof.balance.gt(&0) {
        return Err(ProgramError::InvalidAccountData);
    }

    // Generate bump
    let bump = Pubkey::find_program_address(&[PROOF, signer.key.as_ref()], &crate::id()).1;

    // Realloc data to zero
    drop(proof_data);
    proof_info.realloc(0, false)?;

    // Send lamports to signer
    invoke_signed(
        &solana_program::system_instruction::transfer(
            proof_info.key,
            signer.key,
            proof_info.lamports(),
        ),
        &[proof_info.clone(), signer.clone(), system_program.clone()],
        &[&[PROOF, signer.key.as_ref(), &[bump]]],
    )?;

    // Reassign back to system program
    solana_program::program::invoke_signed(
        &solana_program::system_instruction::assign(proof_info.key, &system_program::id()),
        &[proof_info.clone(), system_program.clone()],
        &[&[PROOF, signer.key.as_ref(), &[bump]]],
    )?;

    Ok(())
}
