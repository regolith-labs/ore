use steel::*;

use crate::state::config_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {
    // The address that can set the admin.
    pub admin: Pubkey,

    /// Number of blocks that can be open for trading at one time.
    pub block_limit: u64,

    // The address that receives fees.
    pub fee_collector: Pubkey,

    // The fee rate taken for each swap.
    pub fee_rate: u64,
}

impl Config {
    pub fn pda() -> (Pubkey, u8) {
        config_pda()
    }
}

account!(OreAccount, Config);
