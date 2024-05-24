use num_enum::IntoPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum OreError {
    #[error("Mining is paused")]
    IsPaused = 0,
    #[error("The epoch has ended and needs reset")]
    NeedsReset = 1,
    #[error("The provided hash is invalid")]
    HashInvalid = 2,
    #[error("The provided hash did not satisfy the minimum required difficulty")]
    HashTooEasy = 3,
    #[error("The claim amount cannot be greater than the claimable rewards")]
    ClaimTooLarge = 4,
    #[error("The clock time is invalid")]
    ClockInvalid = 5,
    #[error("You are trying to submit too soon")]
    Spam = 6,
    #[error("Only one hash may be validated per transaction")]
    TransactionInvalid = 7,
    #[error("The tolerance cannot exceed i64 max value")]
    ToleranceOverflow = 8,
    #[error("The maximum supply has been reached")]
    MaxSupply = 9,
}

impl From<OreError> for ProgramError {
    fn from(e: OreError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
