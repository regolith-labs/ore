mod automation;
mod board;
mod config;
mod miner;
mod round;
mod stake;
mod treasury;

pub use automation::*;
pub use board::*;
pub use config::*;
pub use miner::*;
pub use round::*;
pub use stake::*;
pub use treasury::*;

use crate::consts::*;

/// Account types in the fPOW application
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FpowAccount {
    Automation = 100,
    Config = 101,
    Miner = 103,
    Treasury = 104,
    Board = 105,
    Stake = 108,
    Round = 109,
}

/// Generate the box name for an automation account
pub fn automation_box_name(authority: &[u8; 32]) -> Vec<u8> {
    let mut name = AUTOMATION.to_vec();
    name.extend_from_slice(authority);
    name
}

/// Generate the box name for the board account
pub fn board_box_name() -> Vec<u8> {
    BOARD.to_vec()
}

/// Generate the box name for the config account
pub fn config_box_name() -> Vec<u8> {
    CONFIG.to_vec()
}

/// Generate the box name for a miner account
pub fn miner_box_name(authority: &[u8; 32]) -> Vec<u8> {
    let mut name = MINER.to_vec();
    name.extend_from_slice(authority);
    name
}

/// Generate the box name for a round account
pub fn round_box_name(id: u64) -> Vec<u8> {
    let mut name = ROUND.to_vec();
    name.extend_from_slice(&id.to_le_bytes());
    name
}

/// Generate the box name for a stake account
pub fn stake_box_name(authority: &[u8; 32]) -> Vec<u8> {
    let mut name = STAKE.to_vec();
    name.extend_from_slice(authority);
    name
}

/// Generate the box name for the treasury account
pub fn treasury_box_name() -> Vec<u8> {
    TREASURY.to_vec()
}

/// Fixed-point numeric type for reward calculations
/// Uses 128 bits with 64 bits of fractional precision
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Numeric {
    value: i128,
}

impl Numeric {
    pub const ZERO: Numeric = Numeric { value: 0 };
    const SCALE: i128 = 1 << 64;

    pub fn from_u64(n: u64) -> Self {
        Numeric {
            value: (n as i128) * Self::SCALE,
        }
    }

    pub fn from_fraction(numerator: u64, denominator: u64) -> Self {
        if denominator == 0 {
            return Numeric::ZERO;
        }
        Numeric {
            value: ((numerator as i128) * Self::SCALE) / (denominator as i128),
        }
    }

    pub fn to_u64(&self) -> u64 {
        (self.value / Self::SCALE) as u64
    }

    pub fn to_bytes(&self) -> [u8; 16] {
        self.value.to_le_bytes()
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Numeric {
            value: i128::from_le_bytes(bytes),
        }
    }
}

impl std::ops::Add for Numeric {
    type Output = Numeric;
    fn add(self, other: Numeric) -> Numeric {
        Numeric {
            value: self.value + other.value,
        }
    }
}

impl std::ops::AddAssign for Numeric {
    fn add_assign(&mut self, other: Numeric) {
        self.value += other.value;
    }
}

impl std::ops::Sub for Numeric {
    type Output = Numeric;
    fn sub(self, other: Numeric) -> Numeric {
        Numeric {
            value: self.value - other.value,
        }
    }
}

impl std::ops::Mul for Numeric {
    type Output = Numeric;
    fn mul(self, other: Numeric) -> Numeric {
        Numeric {
            value: (self.value * other.value) / Self::SCALE,
        }
    }
}

impl serde::Serialize for Numeric {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.to_bytes())
    }
}

impl<'de> serde::Deserialize<'de> for Numeric {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = serde::Deserialize::deserialize(deserializer)?;
        if bytes.len() != 16 {
            return Err(serde::de::Error::custom("Invalid Numeric length"));
        }
        let mut arr = [0u8; 16];
        arr.copy_from_slice(&bytes);
        Ok(Numeric::from_bytes(arr))
    }
}
