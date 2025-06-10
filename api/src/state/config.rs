use steel::*;

use crate::state::config_pda;

use super::OreAccount;

// TODO Config stuff

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {}

impl Config {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

account!(OreAccount, Config);
