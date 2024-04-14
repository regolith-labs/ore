use num_enum::IntoPrimitive;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum OreError {
    #[error("The starting time has not passed yet")]
    NotStarted = 0,
    #[error("The epoch has ended and needs reset")]
    NeedsReset = 1,
    #[error("The epoch is active and cannot be reset at this time")]
    ResetTooEarly = 2,
    #[error("The provided hash was invalid")]
    HashInvalid = 3,
    #[error("The bus does not have enough rewards to issue at this time")]
    BusRewardsInsufficient = 4,
    #[error("The claim amount cannot be greater than the claimable rewards")]
    ClaimTooLarge = 5,
}

impl From<OreError> for ProgramError {
    fn from(e: OreError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
