use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::miner_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Automation {
    /// The amount of SOL to deploy on each territory per round.
    pub amount: u64,

    /// The authority of this automation account.
    pub authority: Pubkey,

    /// The amount of SOL this automation has left.
    pub balance: u64,

    /// The executor of this automation account.
    pub executor: Pubkey,

    /// The amount of SOL the executor should receive in fees.
    pub fee: u64,

    /// The strategy this automation uses.
    pub strategy: u64,

    /// The mask of squares this automation should deploy to if preferred strategy.
    /// If strategy is Random, first bit is used to determine how many squares to deploy to.
    pub mask: u64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AutomationStrategy {
    Random = 0,
    Preferred = 1,
}

impl AutomationStrategy {
    pub fn from_u64(value: u64) -> Self {
        Self::try_from(value as u8).unwrap()
    }
}

impl Automation {
    pub fn pda(&self) -> (Pubkey, u8) {
        miner_pda(self.authority)
    }
}

account!(OreAccount, Automation);
