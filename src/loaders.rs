use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, system_program,
};

pub fn load_signer<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    Ok(info)
}

pub fn load_pda<'a, 'info>(
    info: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
    writable: bool,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    let key = Pubkey::create_program_address(seeds, &crate::id())?;
    if !info.key.eq(&key) {
        return Err(ProgramError::InvalidSeeds);
    }
    if !info.owner.eq(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if writable {
        if !info.is_writable {
            return Err(ProgramError::InvalidAccountData);
        }
    }
    Ok(info)
}

pub fn load_uninitialized_pda<'a, 'info>(
    info: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    let key = Pubkey::create_program_address(seeds, &crate::id())?;
    if !info.key.eq(&key) {
        return Err(ProgramError::InvalidSeeds);
    }
    load_uninitialized_account(info)
}

pub fn load_uninitialized_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    if !info.data_is_empty() || !info.owner.eq(&system_program::id()) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    Ok(info)
}

pub fn load_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.key.eq(&key) {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(info)
}
