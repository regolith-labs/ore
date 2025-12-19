use steel::*;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum FpowError {
    #[error("Amount too small")]
    AmountTooSmall = 0,

    #[error("Not authorized")]
    NotAuthorized = 1,
}

error!(FpowError);
