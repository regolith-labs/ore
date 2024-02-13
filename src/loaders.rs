use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    syscalls, system_program,
};
use spl_token::state::Mint;

use crate::{
    state::{Bus, Proof},
    BUS_COUNT, MINT_ADDRESS, TREASURY_ADDRESS,
};

pub fn load_signer<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
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

pub fn load_bus<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.owner.eq(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let bus_data = info.data.borrow();
    let bus = bytemuck::try_from_bytes::<Bus>(&bus_data).unwrap();

    if !(0..BUS_COUNT).contains(&(bus.id as usize)) {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(info)
}

pub fn load_proof<'a, 'info>(
    info: &'a AccountInfo<'info>,
    signer: &Pubkey,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.owner.eq(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let proof_data = info.data.borrow();
    let proof = bytemuck::try_from_bytes::<Proof>(&proof_data).unwrap();

    if !proof.authority.eq(&signer) {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(info)
}

pub fn load_treasury<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.owner.eq(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    if !info.key.eq(&TREASURY_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }

    Ok(info)
}

pub fn load_mint<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.owner.eq(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let mint_data = info.data.borrow();
    if Mint::unpack_unchecked(&mint_data).is_err() {
        return Err(ProgramError::InvalidAccountData);
    }

    if !info.key.eq(&MINT_ADDRESS) {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(info)
}

pub fn load_token_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if !info.owner.eq(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let account_data = info.data.borrow();
    let account = spl_token::state::Account::unpack_unchecked(&account_data)
        .or(Err(ProgramError::InvalidAccountData))?;

    if !account.mint.eq(mint) {
        return Err(ProgramError::InvalidAccountData);
    }
    if !account.owner.eq(owner) {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(info)
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
