use serde::{Deserialize, Serialize};

use crate::state::round_box_name;

/// Round state - tracks a single game round
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Round {
    /// The round number.
    pub id: u64,

    /// The amount of ALGO deployed in each square (in microalgos).
    pub deployed: [u64; 25],

    /// The block seed from Algorand VRF, used for random number generation.
    pub block_seed: [u8; 32],

    /// The count of miners on each square.
    pub count: [u64; 25],

    /// The Algorand round at which claims for this round account end.
    pub expires_at: u64,

    /// The amount of fPOW in the motherlode.
    pub motherlode: u64,

    /// The account to which minimum balance should be returned when this account is closed.
    pub rent_payer: [u8; 32],

    /// The top miner of the round (Algorand address bytes).
    pub top_miner: [u8; 32],

    /// The amount of fPOW to distribute to the top miner.
    pub top_miner_reward: u64,

    /// The total amount of ALGO deployed in the round (in microalgos).
    pub total_deployed: u64,

    /// The total number of unique miners that played in the round.
    pub total_miners: u64,

    /// The total amount of ALGO put in the fPOW vault (in microalgos).
    pub total_vaulted: u64,

    /// The total amount of ALGO won by miners for the round (in microalgos).
    pub total_winnings: u64,
}

impl Round {
    pub fn box_name(&self) -> Vec<u8> {
        round_box_name(self.id)
    }

    /// Generate random number from block seed
    pub fn rng(&self) -> Option<u64> {
        if self.block_seed == [0; 32] || self.block_seed == [u8::MAX; 32] {
            return None;
        }
        let r1 = u64::from_le_bytes(self.block_seed[0..8].try_into().unwrap());
        let r2 = u64::from_le_bytes(self.block_seed[8..16].try_into().unwrap());
        let r3 = u64::from_le_bytes(self.block_seed[16..24].try_into().unwrap());
        let r4 = u64::from_le_bytes(self.block_seed[24..32].try_into().unwrap());
        let r = r1 ^ r2 ^ r3 ^ r4;
        Some(r)
    }

    pub fn winning_square(&self, rng: u64) -> usize {
        (rng % 25) as usize
    }

    pub fn top_miner_sample(&self, rng: u64, winning_square: usize) -> u64 {
        if self.deployed[winning_square] == 0 {
            return 0;
        }
        rng.reverse_bits() % self.deployed[winning_square]
    }

    pub fn calculate_total_winnings(&self, winning_square: usize) -> u64 {
        let mut total_winnings = 0;
        for (i, &deployed) in self.deployed.iter().enumerate() {
            if i != winning_square {
                total_winnings += deployed;
            }
        }
        total_winnings
    }

    pub fn is_split_reward(&self, rng: u64) -> bool {
        // One out of four rounds get split rewards.
        let rng = rng.reverse_bits().to_le_bytes();
        let r1 = u16::from_le_bytes(rng[0..2].try_into().unwrap());
        let r2 = u16::from_le_bytes(rng[2..4].try_into().unwrap());
        let r3 = u16::from_le_bytes(rng[4..6].try_into().unwrap());
        let r4 = u16::from_le_bytes(rng[6..8].try_into().unwrap());
        let r = r1 ^ r2 ^ r3 ^ r4;
        r % 2 == 0
    }

    pub fn did_hit_motherlode(&self, rng: u64) -> bool {
        rng.reverse_bits() % 625 == 0
    }
}

impl Default for Round {
    fn default() -> Self {
        Self {
            id: 0,
            deployed: [0u64; 25],
            block_seed: [0u8; 32],
            count: [0u64; 25],
            expires_at: 0,
            motherlode: 0,
            rent_payer: [0u8; 32],
            top_miner: [0u8; 32],
            top_miner_reward: 0,
            total_deployed: 0,
            total_miners: 0,
            total_vaulted: 0,
            total_winnings: 0,
        }
    }
}
