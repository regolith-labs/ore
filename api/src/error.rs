use thiserror::Error;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FpowError {
    #[error("Amount too small")]
    AmountTooSmall = 0,

    #[error("Not authorized")]
    NotAuthorized = 1,

    #[error("Invalid application state")]
    InvalidState = 2,

    #[error("Box not found")]
    BoxNotFound = 3,

    #[error("Insufficient balance")]
    InsufficientBalance = 4,

    #[error("Round not active")]
    RoundNotActive = 5,

    #[error("Invalid round")]
    InvalidRound = 6,

    #[error("Already claimed")]
    AlreadyClaimed = 7,
}

impl From<FpowError> for u32 {
    fn from(e: FpowError) -> Self {
        e as u32
    }
}
