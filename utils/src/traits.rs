use solana_program::program_error::ProgramError;

pub trait AccountDeserialize {
    fn try_from_bytes(data: &[u8]) -> Result<&Self, ProgramError>;
    fn try_from_bytes_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError>;
}

pub trait Discriminator {
    fn discriminator() -> u8;
}
