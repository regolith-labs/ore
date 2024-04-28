use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::PauseArgs, loaders::*, state::Config, utils::AccountDeserialize};

pub fn process_pause<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = PauseArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, config_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_config(config_info, true)?;

    // Validate signer is admin
    let mut config_data = config_info.data.borrow_mut();
    let config = Config::try_from_bytes_mut(&mut config_data)?;
    if config.admin.ne(&signer.key) {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Update paused
    config.paused = args.paused as u32;

    Ok(())
}
