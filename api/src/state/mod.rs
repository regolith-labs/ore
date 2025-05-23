mod block;
mod proof;
mod treasury;
mod wager;

pub use block::*;
pub use proof::*;
pub use treasury::*;
pub use wager::*;

use steel::*;

use crate::consts::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreAccount {
    Proof = 102,
    Treasury = 103,
    Block = 104,
    Wager = 105,
}

pub fn block_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BLOCK], &crate::ID)
}

pub fn proof_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROOF, authority.as_ref()], &crate::id())
}

pub fn wager_pda(round: u64, id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[WAGER, &round.to_le_bytes(), &id.to_le_bytes()],
        &crate::ID,
    )
}

pub fn treasury_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY], &crate::ID)
}
