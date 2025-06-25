use steel::*;

use crate::state::config_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {
    pub admin: Pubkey,
    pub fee_collector: Pubkey,
    pub fee_rate: u64,
}

impl Config {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

account!(OreAccount, Config);
