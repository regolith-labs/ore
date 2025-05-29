mod block;
mod commit;
mod proof;
mod treasury;

pub use block::*;
pub use commit::*;
pub use proof::*;
pub use treasury::*;

use steel::*;

use crate::consts::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreAccount {
    Proof = 102,
    Treasury = 103,
    Block = 104,
    Commit = 105,
}

pub fn block_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BLOCK], &crate::ID)
}

pub fn proof_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROOF, authority.as_ref()], &crate::id())
}

pub fn treasury_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY], &crate::ID)
}

pub fn commit_pda(round: u64, seed: [u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[COMMIT, &round.to_le_bytes(), &seed], &crate::ID)
}
