mod board;
mod config;
mod miner;
mod square;
mod treasury;

pub use board::*;
pub use config::*;
pub use miner::*;
pub use square::*;
pub use treasury::*;

use crate::consts::*;

use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreAccount {
    Config = 101,
    Miner = 103,
    Treasury = 104,

    //
    Board = 105,
    Square = 106,
}

pub fn board_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BOARD], &crate::ID)
}

pub fn config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CONFIG], &crate::ID)
}

pub fn miner_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINER, &authority.to_bytes()], &crate::ID)
}

pub fn square_pda(id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[SQUARE, &id.to_le_bytes()], &crate::ID)
}

pub fn vault_address() -> Pubkey {
    let board_address = board_pda().0;
    spl_associated_token_account::get_associated_token_address(&board_address, &MINT_ADDRESS)
}

pub fn treasury_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY], &crate::ID)
}

// pub fn treasury_tokens_address() -> Pubkey {
//     spl_associated_token_account::get_associated_token_address(&TREASURY_ADDRESS, &MINT_ADDRESS)
// }
