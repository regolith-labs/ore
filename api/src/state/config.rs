use steel::*;

use crate::state::config_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {
    // The address that can set the admin.
    pub admin: Pubkey,

    // The last boost timestamp.
    pub last_boost: i64,

    // The minimum amount of SOL that can be deploy.
    pub min_deploy_amount: u64,

    // The address that receives fees.
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
