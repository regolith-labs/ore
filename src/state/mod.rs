mod bus;
mod hash;
mod proof;
mod treasury;

pub use bus::*;
pub use hash::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
pub use proof::*;
pub use treasury::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AccountDiscriminator {
    Bus = 100,
    Proof = 101,
    Treasury = 102,
}

pub trait Discriminator {
    fn discriminator() -> AccountDiscriminator;
}
