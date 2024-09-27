mod bus;
mod config;
mod proof;
mod treasury;

pub use bus::*;
pub use config::*;
pub use proof::*;
pub use treasury::*;

use steel::*;

use crate::consts::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreAccount {
    Bus = 100,
    Config = 101,
    Proof = 102,
    Treasury = 103,
}

/// Fetch the PDA of a bus account.
pub fn bus_pda(id: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BUS, &[id]], &crate::id())
}

/// Derive the PDA of the config account.
pub fn config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CONFIG], &crate::id())
}

/// Derive the PDA of a proof account.
pub fn proof_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROOF, authority.as_ref()], &crate::id())
}

/// Derive the PDA of the treasury account.
pub fn treasury_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY], &crate::id())
}
