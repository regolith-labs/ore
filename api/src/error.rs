use steel::*;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum OreError {
    #[error("Amount too small")]
    AmountTooSmall = 0,

    #[error("Not authorized")]
    NotAuthorized = 1,
}

error!(OreError);
