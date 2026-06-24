use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{config_pda, OreAccountV4};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Config {
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

account!(OreAccountV4, Config);

// impl Config {
//     pub fn admin(&self) -> Pubkey {
//         match self {
//             Config::V1(c) => c.admin,
//             Config::V4(c) => c.admin.authority,
//         }
//     }

//     pub fn buffer_a(&self) -> [u8; 32] {
//         match self {
//             Config::V1(c) => c.buffer_a,
//             Config::V4(_) => [0; 32],
//         }
//     }

//     pub fn buffer_b(&self) -> [u8; 32] {
//         match self {
//             Config::V1(c) => c.buffer_b,
//             Config::V4(_) => [0; 32],
//         }
//     }

//     pub fn buffer_c(&self) -> [u8; 32] {
//         match self {
//             Config::V1(c) => c.buffer_c,
//             Config::V4(_) => [0; 32],
//         }
//     }

//     pub fn buffer_d(&self) -> [u8; 32] {
//         match self {
//             Config::V1(c) => c.buffer_d,
//             Config::V4(_) => [0; 32],
//         }
//     }

//     pub fn buffer_e(&self) -> [u8; 8] {
//         match self {
//             Config::V1(c) => c.buffer_e,
//             Config::V4(_) => [0; 8],
//         }
//     }
// }
