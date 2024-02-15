use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    instruction::UpdateDifficultyArgs, loaders::*, state::Treasury, utils::AccountDeserialize,
};

pub fn process_update_difficulty<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = UpdateDifficultyArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, treasury_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_treasury(treasury_info, true)?;

    // Validate admin signer
    let mut treasury_data = treasury_info.data.borrow_mut();
    let mut treasury = Treasury::try_from_bytes_mut(&mut treasury_data)?;
    if !treasury.admin.eq(&signer.key) {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Update admin
    treasury.difficulty = args.new_difficulty;

    Ok(())
}
