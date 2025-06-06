use steel::*;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum OreError {
    #[error("Placeholder error")]
    Dummy = 0,
}

error!(OreError);
