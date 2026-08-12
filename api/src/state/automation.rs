use ore_mint_api::consts::ONE_ORE;
use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{automation_pda, OreAccount};

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

    /// Min motherlode amount (whole ORE units). Deploy blocked if motherlode is below this.
    /// Default: 0 (no lower bound).
    pub min_motherlode: u16,

    /// Max motherlode amount (whole ORE units). Deploy blocked if motherlode exceeds this.
    /// Default: u64::MAX (no upper bound).
    pub max_motherlode: u16,

    /// Number of split tiles to target.
    /// Default: u16::MAX (no preference).
    pub split_tiles: u16,

    /// Number of solo tiles to target.
    /// Default: u16::MAX (no preference).
    pub solo_tiles: u16,

    /// Unused buffer space.
    pub _buffer: u64,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum AutomationStrategy {
    Random = 0,
    Preferred = 1,
    Discretionary = 2,
    DiscretionaryBps = 3,
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
            max_motherlode: u16::MAX,
            split_tiles: 0,
            solo_tiles: 0,
            _buffer: 0,
        }
    }
}

impl AutomationConditions {
    pub fn to_bytes(&self) -> [u8; 24] {
        let mut bytes = [0; 24];
        bytes[0..8].copy_from_slice(&self.max_production_cost.to_le_bytes());
        bytes[8..10].copy_from_slice(&self.min_motherlode.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.max_motherlode.to_le_bytes());
        bytes[12..14].copy_from_slice(&self.split_tiles.to_le_bytes());
        bytes[14..16].copy_from_slice(&self.solo_tiles.to_le_bytes());
        bytes[16..24].copy_from_slice(&self._buffer.to_le_bytes());
        bytes
    }

    pub fn from_bytes(bytes: [u8; 24]) -> Self {
        Self {
            max_production_cost: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            min_motherlode: u16::from_le_bytes(bytes[8..10].try_into().unwrap()),
            max_motherlode: u16::from_le_bytes(bytes[10..12].try_into().unwrap()),
            split_tiles: u16::from_le_bytes(bytes[12..14].try_into().unwrap()),
            solo_tiles: u16::from_le_bytes(bytes[14..16].try_into().unwrap()),
            _buffer: u64::from_le_bytes(bytes[16..24].try_into().unwrap()),
        }
    }
}

impl Automation {
    pub fn pda(&self) -> (Pubkey, u8) {
        automation_pda(self.authority)
    }

    pub fn min_fee(&self, deploy_amount: u64) -> u64 {
        if self.strategy == AutomationStrategy::DiscretionaryBps as u64 {
            ((deploy_amount as u128 * self.fee as u128) / crate::consts::DENOMINATOR_BPS as u128) as u64
        } else {
            self.fee
        }
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
