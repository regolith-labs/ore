use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::config_pda;

use super::FpowAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Config {
    /// The address that can update the config.
    pub admin: Pubkey,

    /// Buffer a (placeholder)
    pub buffer_a: [u8; 32],

    /// Buffer b (placeholder)
    pub buffer_b: [u8; 32],

    /// Buffer c (placeholder)
    pub buffer_c: [u8; 32],

    /// Buffer d (placeholder)
    pub buffer_d: [u8; 32],

    /// Buffer e (placeholder)
    pub buffer_e: [u8; 8],
}

impl Config {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

account!(FpowAccount, Config);
