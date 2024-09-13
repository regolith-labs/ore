mod bus;
mod config;
mod proof;
mod proof_v2;
mod treasury;

pub use bus::*;
pub use config::*;
pub use proof::*;
pub use proof_v2::*;
pub use treasury::*;

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AccountDiscriminator {
    Bus = 100,
    Config = 101,
    Proof = 102,
    Treasury = 103,
    ProofV2 = 104,
    WoodBus = 105,
    WoodConfig = 106,
}
