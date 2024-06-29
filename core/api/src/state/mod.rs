mod bus;
mod config;
mod proof;
mod treasury;

pub use bus::*;
pub use config::*;
pub use proof::*;
pub use treasury::*;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AccountDiscriminator {
    Bus = 100,
    Config = 101,
    Proof = 102,
    Treasury = 103,
}
