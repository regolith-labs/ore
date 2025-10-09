use steel::*;

use crate::state::{config_pda, OreAccountOLD};

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {
    /// The address that can update the config.
    pub admin: Pubkey,

    /// The adress with authority to call bury.
    pub bury_authority: Pubkey,

    /// The address that receives admin fees.
    pub fee_collector: Pubkey,

    /// The last boost timestamp.
    pub last_boost: i64,

    /// Whether seeker activation is enabled.
    pub is_seeker_activation_enabled: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ConfigOLD {
    /// The address that can update the config.
    pub admin: Pubkey,

    /// The last boost timestamp.
    pub last_boost: i64,

    /// Whether seeker activation is enabled.
    pub is_seeker_activation_enabled: u64,

    /// The address that receives admin fees.
    pub fee_collector: Pubkey,

    // The fee rate taken for each swap.
    #[deprecated(since = "1.0.0", note = "Unused")]
    pub fee_rate: u64,
}

impl Config {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

account!(OreAccount, Config);
account!(OreAccountOLD, ConfigOLD);
