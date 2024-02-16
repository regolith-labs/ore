use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    system_program,
};
use spl_token::state::Mint;

use crate::{
    state::{Bus, Proof},
    utils::AccountDeserialize,
    BUS_ADDRESSES, BUS_COUNT, MINT_ADDRESS, TREASURY_ADDRESS,
};

pub fn load_signer<'a, 'info>(info: &'a AccountInfo<'info>) -> Result<(), ProgramError> {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

pub fn load_bus<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let bus_data = info.data.borrow();
    let bus = Bus::try_from_bytes(&bus_data)?;

    if bus.id.ge(&(BUS_COUNT as u64)) {
        return Err(ProgramError::InvalidAccountData);
    }

    if info.key.ne(&BUS_ADDRESSES[bus.id as usize]) {
        return Err(ProgramError::InvalidSeeds);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

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

pub fn load_treasury<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    if info.key.ne(&TREASURY_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

pub fn load_mint<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let mint_data = info.data.borrow();
    if Mint::unpack_unchecked(&mint_data).is_err() {
        return Err(ProgramError::InvalidAccountData);
    }

    if info.key.ne(&MINT_ADDRESS) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

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
    let account = spl_token::state::Account::unpack_unchecked(&account_data)
        .or(Err(ProgramError::InvalidAccountData))?;

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

pub fn load_uninitialized_pda<'a, 'info>(
    info: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
) -> Result<(), ProgramError> {
    let key = Pubkey::create_program_address(seeds, &crate::id())?;
    if info.key.ne(&key) {
        return Err(ProgramError::InvalidSeeds);
    }
    load_uninitialized_account(info)
}

pub fn load_uninitialized_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<(), ProgramError> {
    if info.owner.ne(&system_program::id()) {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if !info.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

pub fn load_sysvar<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<(), ProgramError> {
    load_account(info, key, false)
}

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

pub fn load_program<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<(), ProgramError> {
    if info.key.ne(&key) {
        return Err(ProgramError::InvalidAccountData);
    }

    if !info.executable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}
