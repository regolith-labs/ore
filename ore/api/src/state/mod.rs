mod block;
mod config;
mod market;
mod miner;
mod permit;
mod treasury;

pub use block::*;
pub use config::*;
pub use market::*;
pub use miner::*;
pub use permit::*;
pub use treasury::*;

use crate::consts::*;

use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreAccount {
    Block = 100,
    Config = 101,
    Market = 102,
    Miner = 103,
    Permit = 104,
    Treasury = 105,
}

pub fn block_pda(id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BLOCK, &id.to_le_bytes()], &crate::ID)
}

pub fn config_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[CONFIG], &crate::ID)
}

pub fn market_pda(id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MARKET, &id.to_le_bytes()], &crate::ID)
}

pub fn miner_pda(authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINER, &authority.to_bytes()], &crate::ID)
}

pub fn mint_pda(id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT, &id.to_le_bytes()], &crate::ID)
}

pub fn permit_pda(authority: Pubkey, block_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[PERMIT, &authority.to_bytes(), &block_id.to_le_bytes()],
        &crate::ID,
    )
}

pub fn treasury_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[TREASURY], &crate::ID)
}
