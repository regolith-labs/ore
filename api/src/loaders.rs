use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    system_program, sysvar,
};
use spl_token::state::Mint;

use crate::{
    consts::*,
    state::{Bus, Config, Proof, Treasury},
    utils::{AccountDeserialize, Discriminator},
};

/// Errors if:
/// - Account is not a signer.
pub fn load_signer<'a, 'info>(info: &'a AccountInfo<'info>) -> Result<(), ProgramError> {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Address does not match the expected bus address.
/// - Data is empty.
/// - Data cannot deserialize into a bus account.
/// - Bus ID does not match the expected ID.
/// - Expected to be writable, but is not.
pub fn load_bus<'a, 'info>(
    info: &'a AccountInfo<'info>,
    id: u64,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.key.ne(&BUS_ADDRESSES[id as usize]) {
        return Err(ProgramError::InvalidSeeds);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let bus_data = info.data.borrow();
    let bus = Bus::try_from_bytes(&bus_data)?;

    if bus.id.ne(&id) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Data is empty.
/// - Data cannot deserialize into a bus account.
/// - Bus ID is not in the expected range.
/// - Address is not in set of valid bus address.
/// - Expected to be writable, but is not.
pub fn load_any_bus<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    if info.data.borrow()[0].ne(&(Bus::discriminator() as u8)) {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }

    if !BUS_ADDRESSES.contains(info.key) {
        return Err(ProgramError::InvalidSeeds);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Address does not match the expected address.
/// - Data is empty.
/// - Data cannot deserialize into a config account.
/// - Expected to be writable, but is not.
pub fn load_config<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.key.ne(&CONFIG_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    if info.data.borrow()[0].ne(&(Config::discriminator() as u8)) {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Data is empty.
/// - Data cannot deserialize into a proof account.
/// - Proof authority does not match the expected address.
/// - Expected to be writable, but is not.
pub fn load_proof<'a, 'info>(
    info: &'a AccountInfo<'info>,
    authority: &Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let proof_data = info.data.borrow();
    let proof = Proof::try_from_bytes(&proof_data)?;

    if proof.authority.ne(&authority) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Data is empty.
/// - Data cannot deserialize into a proof account.
/// - Proof miner does not match the expected address.
/// - Expected to be writable, but is not.
pub fn load_proof_with_miner<'a, 'info>(
    info: &'a AccountInfo<'info>,
    miner: &Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let proof_data = info.data.borrow();
    let proof = Proof::try_from_bytes(&proof_data)?;

    if proof.miner.ne(&miner) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Data is empty.
/// - Data cannot deserialize into a proof account.
/// - Expected to be writable, but is not.
pub fn load_any_proof<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    if info.data.borrow()[0].ne(&(Proof::discriminator() as u8)) {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Address does not match the expected address.
/// - Data is empty.
/// - Data cannot deserialize into a treasury account.
/// - Expected to be writable, but is not.
pub fn load_treasury<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.key.ne(&TREASURY_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    if info.data.borrow()[0].ne(&(Treasury::discriminator() as u8)) {
        return Err(solana_program::program_error::ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Address does not match the expected treasury tokens address.
/// - Cannot load as a token account
pub fn load_treasury_tokens<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.key.ne(&TREASURY_TOKENS_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }

    load_token_account(info, Some(&TREASURY_ADDRESS), &MINT_ADDRESS, is_writable)
}

/// Errors if:
/// - Owner is not SPL token program.
/// - Address does not match the expected mint address.
/// - Data is empty.
/// - Data cannot deserialize into a mint account.
/// - Expected to be writable, but is not.
pub fn load_mint<'a, 'info>(
    info: &'a AccountInfo<'info>,
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
/// - Data cannot deserialize into a token account.
/// - Token account owner does not match the expected owner address.
/// - Token account mint does not match the expected mint address.
/// - Expected to be writable, but is not.
pub fn load_token_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
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
/// - Address does not match PDA derived from provided seeds.
/// - Cannot load as an uninitialized account.
pub fn load_uninitialized_pda<'a, 'info>(
    info: &'a AccountInfo<'info>,
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
pub fn load_system_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
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
pub fn load_sysvar<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<(), ProgramError> {
    if info.owner.ne(&sysvar::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    load_account(info, key, false)
}

/// Errors if:
/// - Address does not match the expected value.
/// - Expected to be writable, but is not.
pub fn load_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
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
pub fn load_program<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<(), ProgramError> {
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
pub fn load_any<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
