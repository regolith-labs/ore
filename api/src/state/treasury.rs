use bytemuck::{Pod, Zeroable};
use ore_utils::*;
use solana_program::pubkey::Pubkey;

use crate::consts::TREASURY;

use super::OreAccount;

/// Treasury is a singleton account which is the mint authority for the ORE token and the authority of
/// the program's global token account.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {}

/// Derive the PDA of the treasury account.
pub fn treasury_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY], &crate::id())
}

account!(OreAccount, Treasury);
