use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{config_pda, OreAccountV4};

use super::OreAccount;

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

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct ConfigV4 {
    /// The admin config.
    pub admin: AdminConfig,

    /// The protocol config.
    pub protocol: ProtocolConfig,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct AdminConfig {
    /// The authority of this admin config.
    pub authority: Pubkey,

    /// The address of the admin fee collector.
    pub fee_collector: Pubkey,

    /// The admin fee rate.
    pub fee_rate: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// The authority of this protocol config.
    pub authority: Pubkey,

    /// The address of the protocol fee collector.
    pub fee_collector: Pubkey,

    /// The protocol fee rate.
    pub fee_rate: u64,

    /// The number of slots in the intermission period.
    pub intermission_slots: u64,

    /// The number of slots in a round.
    pub round_slots: u64,

    /// The address of the var account.
    pub entropy_var_address: Pubkey,

    /// The entropy program id.
    pub entropy_program_id: Pubkey,
}

impl Config {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

impl ConfigV4 {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

account!(OreAccount, Config);
account!(OreAccountV4, ConfigV4);
