use num_enum::IntoPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum OreError {
    #[error("The epoch is still active and cannot be reset")]
    EpochActive = 0,
    #[error("The epoch has expired and needs reset")]
    EpochExpired = 1,
    #[error("The provided hash was invalid")]
    InvalidHash = 2,
    #[error("The provided hash does not satisfy the difficulty requirement")]
    InsufficientHashDifficulty = 3,
    #[error("The bus has insufficient rewards to mine at this time")]
    InsufficientBusRewards = 4,
    #[error("The claim amount cannot be larger than the claimable rewards")]
    InvalidClaimAmount = 5,
}

impl From<OreError> for ProgramError {
    fn from(e: OreError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
