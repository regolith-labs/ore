use steel::*;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum OreError {
    #[error("Placeholder error")]
    Dummy = 0,

    #[error("Insufficient vault reserves")]
    InsufficientVaultReserves = 1,

    #[error("Invariant violation")]
    InvariantViolation = 2,

    #[error("Insufficient liquidity")]
    InsufficientLiquidity = 3,
}

error!(OreError);
