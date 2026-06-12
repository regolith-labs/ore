use ore_mint_api::consts::ONE_ORE;
use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{automation_pda, OreAccountV4};

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

    /// Whether or not to auto-reload SOL winnings into the automation balance.
    pub reload: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct AutomationV4 {
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

    /// Whether or not to auto-reload SOL winnings into the automation balance.
    pub reload: u64,

    /// The total SOL spent (lost to fees) by this automation.
    pub total_sol_spent: u64,

    /// The total ORE earned by this automation.
    pub total_ore_earned: u64,

    /// Conditions that must be met for the automation to deploy.
    pub conditions: AutomationConditions,
}

/// Conditions that gate whether an automation deploys in a given round.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct AutomationConditions {
    /// Max production cost EMA (lamports per whole ORE). Deploy blocked if EMA exceeds this.
    /// Default: u64::MAX (no upper bound).
    pub max_production_cost: u64,

    /// Min motherlode amount (ORE units). Deploy blocked if motherlode is below this.
    /// Default: 0 (no lower bound).
    pub min_motherlode: u64,

    /// Max motherlode amount (ORE units). Deploy blocked if motherlode exceeds this.
    /// Default: u64::MAX (no upper bound).
    pub max_motherlode: u64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AutomationStrategy {
    Random = 0,
    Preferred = 1,
    Discretionary = 2,
}

impl AutomationStrategy {
    pub fn from_u64(value: u64) -> Self {
        Self::try_from(value as u8).unwrap()
    }
}

impl Default for AutomationConditions {
    fn default() -> Self {
        Self {
            max_production_cost: u64::MAX,
            min_motherlode: 0,
            max_motherlode: u64::MAX,
        }
    }
}

impl AutomationV4 {
    pub fn pda(&self) -> (Pubkey, u8) {
        automation_pda(self.authority)
    }

    pub fn production_cost(&self) -> u64 {
        if self.total_ore_earned == 0 {
            return 0;
        }
        ((self.total_sol_spent as u128) * (ONE_ORE as u128) / (self.total_ore_earned as u128))
            as u64
    }
}

account!(OreAccount, Automation);
account!(OreAccountV4, AutomationV4);

pub enum AutomationAccount {
    Automation(Automation),
    AutomationV4(AutomationV4),
}

impl AutomationAccount {
    pub fn amount(&self) -> u64 {
        match self {
            AutomationAccount::Automation(a) => a.amount,
            AutomationAccount::AutomationV4(a) => a.amount,
        }
    }

    pub fn authority(&self) -> Pubkey {
        match self {
            AutomationAccount::Automation(a) => a.authority,
            AutomationAccount::AutomationV4(a) => a.authority,
        }
    }

    pub fn balance(&self) -> u64 {
        match self {
            AutomationAccount::Automation(a) => a.balance,
            AutomationAccount::AutomationV4(a) => a.balance,
        }
    }

    pub fn executor(&self) -> Pubkey {
        match self {
            AutomationAccount::Automation(a) => a.executor,
            AutomationAccount::AutomationV4(a) => a.executor,
        }
    }

    pub fn fee(&self) -> u64 {
        match self {
            AutomationAccount::Automation(a) => a.fee,
            AutomationAccount::AutomationV4(a) => a.fee,
        }
    }

    pub fn strategy(&self) -> u64 {
        match self {
            AutomationAccount::Automation(a) => a.strategy,
            AutomationAccount::AutomationV4(a) => a.strategy,
        }
    }

    pub fn mask(&self) -> u64 {
        match self {
            AutomationAccount::Automation(a) => a.mask,
            AutomationAccount::AutomationV4(a) => a.mask,
        }
    }

    pub fn reload(&self) -> u64 {
        match self {
            AutomationAccount::Automation(a) => a.reload,
            AutomationAccount::AutomationV4(a) => a.reload,
        }
    }
}
