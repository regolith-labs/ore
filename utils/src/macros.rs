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
        impl $crate::AccountDeserialize for $struct_name {
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

#[macro_export]
macro_rules! account {
    ($discriminator_name:ident, $struct_name:ident) => {
        $crate::impl_to_bytes!($struct_name);
        $crate::impl_account_from_bytes!($struct_name);

        impl $crate::Discriminator for $struct_name {
            fn discriminator() -> u8 {
                $discriminator_name::$struct_name.into()
            }
        }
    };
}

#[macro_export]
macro_rules! instruction {
    ($struct_name:ident) => {
        $crate::impl_to_bytes!($struct_name);
        $crate::impl_instruction_from_bytes!($struct_name);
    };
}
