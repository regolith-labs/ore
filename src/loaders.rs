use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_memory::sol_memcmp,
    program_pack::Pack, pubkey::Pubkey, system_program,
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
    if sol_memcmp(info.key.as_ref(), key.as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidSeeds);
    }
    load_uninitialized_account(info)
}

pub fn load_bus<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if sol_memcmp(info.owner.as_ref(), crate::id().as_ref(), 32) != 0 {
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
    if sol_memcmp(info.owner.as_ref(), crate::id().as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let proof_data = info.data.borrow();
    let proof = bytemuck::try_from_bytes::<Proof>(&proof_data).unwrap();

    // if !proof.authority.eq(&signer) {
    if sol_memcmp(proof.authority.as_ref(), signer.as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(info)
}

pub fn load_treasury<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if sol_memcmp(info.owner.as_ref(), crate::id().as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    if sol_memcmp(info.key.as_ref(), TREASURY_ADDRESS.as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidSeeds);
    }

    Ok(info)
}

pub fn load_mint<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if sol_memcmp(info.owner.as_ref(), spl_token::id().as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let mint_data = info.data.borrow();
    if Mint::unpack_unchecked(&mint_data).is_err() {
        return Err(ProgramError::InvalidAccountData);
    }

    if sol_memcmp(info.key.as_ref(), MINT_ADDRESS.as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(info)
}

pub fn load_token_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
    owner: &Pubkey,
    mint: &Pubkey,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if sol_memcmp(info.owner.as_ref(), spl_token::id().as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountOwner);
    }
    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let account_data = info.data.borrow();
    let account = spl_token::state::Account::unpack_unchecked(&account_data)
        .or(Err(ProgramError::InvalidAccountData))?;

    if sol_memcmp(account.mint.as_ref(), mint.as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountData);
    }
    if sol_memcmp(account.owner.as_ref(), owner.as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(info)
}

pub fn load_uninitialized_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if sol_memcmp(info.owner.as_ref(), system_program::id().as_ref(), 32) != 0 {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !info.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    if !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(info)
}

pub fn load_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<&'a AccountInfo<'info>, ProgramError> {
    if sol_memcmp(info.key.as_ref(), key.as_ref(), 32) != 0 {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(info)
}
