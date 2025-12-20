use serde::{Deserialize, Serialize};

use crate::state::miner_box_name;

/// Automation account for automated mining
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Automation {
    /// The amount of ALGO to deploy on each territory per round.
    pub amount: u64,

    /// The authority of this automation account (Algorand address bytes).
    pub authority: [u8; 32],

    /// The amount of ALGO this automation has left (in microalgos).
    pub balance: u64,

    /// The executor of this automation account (Algorand address bytes).
    pub executor: [u8; 32],

    /// The amount of ALGO the executor should receive in fees.
    pub fee: u64,

    /// The strategy this automation uses.
    pub strategy: u64,

    /// The mask of squares this automation should deploy to if preferred strategy.
    /// If strategy is Random, first bit is used to determine how many squares to deploy to.
    pub mask: u64,

    /// Whether or not to auto-reload ALGO winnings into the automation balance.
    pub reload: u64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AutomationStrategy {
    Random = 0,
    Preferred = 1,
    Discretionary = 2,
}

impl AutomationStrategy {
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            0 => Some(AutomationStrategy::Random),
            1 => Some(AutomationStrategy::Preferred),
            2 => Some(AutomationStrategy::Discretionary),
            _ => None,
        }
    }
}

impl Automation {
    pub fn box_name(&self) -> Vec<u8> {
        miner_box_name(&self.authority)
    }
}

impl Default for Automation {
    fn default() -> Self {
        Self {
            amount: 0,
            authority: [0u8; 32],
            balance: 0,
            executor: [0u8; 32],
            fee: 0,
            strategy: 0,
            mask: 0,
            reload: 0,
        }
    }
}
