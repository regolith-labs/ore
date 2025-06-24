use steel::*;

use crate::state::stake_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Stake {
    /// The authority of the miner account.
    pub authority: Pubkey,

    /// The ID of the block this collateral is associated with.
    pub block_id: u64,

    /// The amount of ORE this miner has deposited as collateral for trading.
    pub collateral: u64,

    /// The amount of ORE this miner has spent on hashpower in this market.
    pub spend: u64,
}

impl Stake {
    pub fn pda(&self) -> (Pubkey, u8) {
        stake_pda(self.authority, self.block_id)
    }
}

account!(OreAccount, Stake);
