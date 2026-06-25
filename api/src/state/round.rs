use serde::{Deserialize, Serialize};
use steel::*;

use crate::state::{round_pda, OreAccountV4};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Round {
    /// The round number.
    pub id: u64,

    /// The amount of SOL deployed in each square.
    /// TODO: Rename to sol.
    pub deployed: [u64; 25],

    /// The amount of mass deployed in each square.
    pub mass: [u64; 25],

    /// The number of unique miners on each square.
    /// TODO rename to miners.
    pub count: [u64; 25],

    /// The entropy value.
    /// TODO: Rename to entropy.
    pub slot_hash: [u8; 32],

    /// The slot after which this account may be closed.
    /// TODO: Rename to closes_at.
    pub expires_at: u64,

    /// The amount of ORE distributed as the motherlode reward.
    pub motherlode: u64,

    /// The account to which rent should be returned to when this account is closed.
    pub rent_payer: Pubkey,

    /// The amount of ORE to distribute to miners.
    pub rewards: [u64; 25],

    /// The total SOL collected by the protocol.
    /// TODO: Rename to protocol_fee.
    pub total_vaulted: u64,

    /// The total SOL returned to miners.
    /// TODO: Rename to total_returned.
    pub total_winnings: u64,

    /// The total number of unique miners that played in the round.
    /// TODO rename to unique_miners.
    pub total_miners: u64,

    /// The winner of the solo reward.
    /// TODO: Rename to winner.
    pub top_miner: Pubkey,
}

impl Round {
    pub fn pda(&self) -> (Pubkey, u8) {
        round_pda(self.id)
    }

    pub fn rng(&self) -> Option<u64> {
        if self.slot_hash == [0; 32] || self.slot_hash == [u8::MAX; 32] {
            return None;
        }
        let r1 = u64::from_le_bytes(self.slot_hash[0..8].try_into().unwrap());
        let r2 = u64::from_le_bytes(self.slot_hash[8..16].try_into().unwrap());
        let r3 = u64::from_le_bytes(self.slot_hash[16..24].try_into().unwrap());
        let r4 = u64::from_le_bytes(self.slot_hash[24..32].try_into().unwrap());
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

    pub fn total_deployed(&self) -> u64 {
        self.deployed.iter().sum()
    }

    pub fn top_miner_reward(&self) -> u64 {
        self.rewards.iter().sum()
    }
}

account!(OreAccountV4, Round);
