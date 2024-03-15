use num_enum::IntoPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum OreError {
    #[error("The start time has not passed yet")]
    NotStarted = 0,
    #[error("The epoch has ended and needs reset")]
    NeedsReset = 1,
    #[error("The epoch is active and cannot be reset at this time")]
    InvalidReset = 2,
    #[error("The provided hash was invalid")]
    InvalidHash = 3,
    #[error("The provided hash does not satisfy the difficulty requirement")]
    InsufficientHashDifficulty = 4,
    #[error("The bus has insufficient rewards to mine at this time")]
    InsufficientBusRewards = 5,
    #[error("The claim amount cannot be larger than the claimable rewards")]
    InvalidClaimAmount = 6,
}

impl From<OreError> for ProgramError {
    fn from(e: OreError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
