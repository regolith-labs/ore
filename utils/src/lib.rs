use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey, rent::Rent, sysvar::Sysvar,
};

/// Creates a new pda
#[inline(always)]
pub fn create_pda<'a, 'info>(
    target_account: &'a AccountInfo<'info>,
    owner: &Pubkey,
    space: usize,
    pda_seeds: &[&[u8]],
    system_program: &'a AccountInfo<'info>,
    payer: &'a AccountInfo<'info>,
) -> ProgramResult {
    let rent = Rent::get()?;
    if target_account.lamports().eq(&0) {
        // If balance is zero, create account
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::create_account(
                payer.key,
                target_account.key,
                rent.minimum_balance(space),
                space as u64,
                owner,
            ),
            &[
                payer.clone(),
                target_account.clone(),
                system_program.clone(),
            ],
            &[pda_seeds],
        )?;
    } else {
        // Otherwise, if balance is nonzero:

        // 1) transfer sufficient lamports for rent exemption
        let rent_exempt_balance = rent
            .minimum_balance(space)
            .saturating_sub(target_account.lamports());
        if rent_exempt_balance.gt(&0) {
            solana_program::program::invoke(
                &solana_program::system_instruction::transfer(
                    payer.key,
                    target_account.key,
                    rent_exempt_balance,
                ),
                &[
                    payer.clone(),
                    target_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }

        // 2) allocate space for the account
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::allocate(target_account.key, space as u64),
            &[target_account.clone(), system_program.clone()],
            &[pda_seeds],
        )?;

        // 3) assign our program as the owner
        solana_program::program::invoke_signed(
            &solana_program::system_instruction::assign(target_account.key, owner),
            &[target_account.clone(), system_program.clone()],
            &[pda_seeds],
        )?;
    }

    Ok(())
}

pub trait Discriminator {
    fn discriminator() -> u8; //AccountDiscriminator;
}

pub trait AccountDeserialize {
    fn try_from_bytes(data: &[u8]) -> Result<&Self, ProgramError>;
    fn try_from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError>;
}

#[macro_export]
macro_rules! impl_to_bytes {
    ($struct_name:ident) => {
        impl $struct_name {
            pub fn to_bytes(&self) -> &[u8] {
                bytemuck::bytes_of(self)
            }
        }
    };
}

#[macro_export]
macro_rules! impl_account_from_bytes {
    ($struct_name:ident) => {
        impl crate::utils::AccountDeserialize for $struct_name {
            fn try_from_bytes(
                data: &[u8],
            ) -> Result<&Self, solana_program::program_error::ProgramError> {
                if Self::discriminator().ne(&data[0]) {
                    return Err(solana_program::program_error::ProgramError::InvalidAccountData);
                }
                bytemuck::try_from_bytes::<Self>(&data[8..]).or(Err(
                    solana_program::program_error::ProgramError::InvalidAccountData,
                ))
            }
            fn try_from_bytes_mut(
                data: &mut [u8],
            ) -> Result<&mut Self, solana_program::program_error::ProgramError> {
                if Self::discriminator().ne(&data[0]) {
                    return Err(solana_program::program_error::ProgramError::InvalidAccountData);
                }
                bytemuck::try_from_bytes_mut::<Self>(&mut data[8..]).or(Err(
                    solana_program::program_error::ProgramError::InvalidAccountData,
                ))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_instruction_from_bytes {
    ($struct_name:ident) => {
        impl $struct_name {
            pub fn try_from_bytes(
                data: &[u8],
            ) -> Result<&Self, solana_program::program_error::ProgramError> {
                bytemuck::try_from_bytes::<Self>(data).or(Err(
                    solana_program::program_error::ProgramError::InvalidInstructionData,
                ))
            }
        }
    };
}
