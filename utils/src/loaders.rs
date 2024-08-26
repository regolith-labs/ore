#[cfg(feature = "spl")]
use solana_program::program_pack::Pack;
use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey, system_program, sysvar,
};
#[cfg(feature = "spl")]
use spl_token::state::Mint;

/// Errors if:
/// - Account is not a signer.
pub fn load_signer(info: &AccountInfo<'_>) -> Result<(), ProgramError> {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

/// Errors if:
/// - Address does not match PDA derived from provided seeds.
/// - Cannot load as an uninitialized account.
pub fn load_uninitialized_pda(
    info: &AccountInfo<'_>,
    seeds: &[&[u8]],
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let pda = Pubkey::find_program_address(seeds, program_id);

    if info.key.ne(&pda.0) {
        return Err(ProgramError::InvalidSeeds);
    }

    if bump.ne(&pda.1) {
        return Err(ProgramError::InvalidSeeds);
    }

    load_system_account(info, true)
}

/// Errors if:
/// - Owner is not the system program.
/// - Data is not empty.
/// - Account is not writable.
pub fn load_system_account(info: &AccountInfo<'_>, is_writable: bool) -> Result<(), ProgramError> {
    if info.owner.ne(&system_program::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !info.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not the sysvar address.
/// - Account cannot load with the expected address.
pub fn load_sysvar(info: &AccountInfo<'_>, key: Pubkey) -> Result<(), ProgramError> {
    if info.owner.ne(&sysvar::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    load_account(info, key, false)
}

/// Errors if:
/// - Address does not match the expected value.
/// - Expected to be writable, but is not.
pub fn load_account(
    info: &AccountInfo<'_>,
    key: Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.key.ne(&key) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Address does not match the expected value.
/// - Account is not executable.
pub fn load_program(info: &AccountInfo<'_>, key: Pubkey) -> Result<(), ProgramError> {
    if info.key.ne(&key) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !info.executable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Account is not writable.
pub fn load_any(info: &AccountInfo<'_>, is_writable: bool) -> Result<(), ProgramError> {
    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not SPL token program.
/// - Address does not match the expected mint address.
/// - Data is empty.
/// - Data cannot deserialize into a mint account.
/// - Expected to be writable, but is not.
#[cfg(feature = "spl")]
pub fn load_mint(
    info: &AccountInfo<'_>,
    address: Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.key.ne(&address) {
        return Err(ProgramError::InvalidSeeds);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    Mint::unpack(&info.data.borrow())?;

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not SPL token program.
/// - Data is empty.
/// - Data cannot deserialize into a mint account.
/// - Expected to be writable, but is not.
#[cfg(feature = "spl")]
pub fn load_any_mint(info: &AccountInfo<'_>, is_writable: bool) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    Mint::unpack(&info.data.borrow())?;

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not SPL token program.
/// - Data is empty.
/// - Data cannot deserialize into a token account.
/// - Token account owner does not match the expected owner address.
/// - Token account mint does not match the expected mint address.
/// - Expected to be writable, but is not.
#[cfg(feature = "spl")]
pub fn load_token_account(
    info: &AccountInfo<'_>,
    owner: Option<&Pubkey>,
    mint: &Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let account_data = info.data.borrow();
    let account = spl_token::state::Account::unpack(&account_data)?;

    if account.mint.ne(&mint) {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(owner) = owner {
        if account.owner.ne(owner) {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not SPL token program.
/// - Data is empty.
/// - Data cannot deserialize into a token account.
/// - Address does not match the expected associated token address.
/// - Expected to be writable, but is not.
#[cfg(feature = "spl")]
pub fn load_associated_token_account(
    info: &AccountInfo<'_>,
    owner: &Pubkey,
    mint: &Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let account_data = info.data.borrow();
    let _ = spl_token::state::Account::unpack(&account_data)?;

    let address = spl_associated_token_account::get_associated_token_address(owner, mint);
    if info.key.ne(&address) {
        return Err(ProgramError::InvalidSeeds);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
