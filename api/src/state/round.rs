use serde::{Deserialize, Serialize};
use solana_program::keccak;
use steel::*;

use crate::state::{round_pda, OreAccount};

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
        rng.reverse_bits() % 500 == 0
    }

    pub fn total_deployed(&self) -> u64 {
        self.deployed.iter().sum()
    }

    pub fn top_miner_reward(&self) -> u64 {
        self.rewards.iter().sum()
    }

    /// Generates a mask that indicates how rewards are distributed on each tile.
    /// The mask is a 32-bit integer where the first 25 bits represent the tiles.
    /// The bits are set to 0 if the reward on that tile is split.
    /// The bits are set to 1 if the reward on that tile is not split.
    /// The mask is generated using a Fisher-Yates shuffle of the rng hash.
    /// The shuffle is done using a Fisher-Yates shuffle for unbiased selection.
    pub fn distribution_mask(&self) -> u32 {
        const BITS: u32 = 10;
        let rng = keccak::hashv(&[self.id.to_le_bytes().as_ref()]);

        // Deterministically select 10 unique indices out of 25 (first 25 bits)
        // using Fisher-Yates shuffle seeded from rng for reproducibility.
        let mut indices: [u8; 25] = [0; 25];
        for i in 0..25 {
            indices[i] = i as u8;
        }

        // Use bytes from the rng hash as randomness source
        let mut randomness = rng.0;
        let mut random_offset = 0;

        // Do a Fisher-Yates shuffle for unbiased selection
        for i in (1..25).rev() {
            // If we've used up all the randomness, rehash (although 32 bytes is plenty for this)
            if random_offset + 2 > randomness.len() {
                // rehash for more randomness (although shouldn't be needed for 25 draws)
                randomness = keccak::hashv(&[&randomness]).0;
                random_offset = 0;
            }
            let mut two_bytes = [0u8; 2];
            two_bytes.copy_from_slice(&randomness[random_offset..random_offset + 2]);
            let r = u16::from_le_bytes(two_bytes);
            let j = (r as usize) % (i + 1);
            indices.swap(i, j);
            random_offset += 2;
        }

        // Set mask bits for the first 10 shuffled indices
        let mut mask: u32 = 0;
        for &idx in &indices[..BITS as usize] {
            mask |= 1 << idx;
        }

        // Only first 25 bits are used, highest 7 bits remain 0
        mask
    }
}

account!(OreAccount, Round);

#[cfg(test)]
mod tests {
    use super::*;

    fn default_round(id: u64) -> Round {
        Round {
            id,
            deployed: [0; 25],
            mass: [0; 25],
            count: [0; 25],
            slot_hash: [0; 32],
            expires_at: 0,
            motherlode: 0,
            rent_payer: Pubkey::default(),
            rewards: [0; 25],
            total_vaulted: 0,
            total_winnings: 0,
            total_miners: 0,
            top_miner: Pubkey::default(),
        }
    }

    #[test]
    fn test_distribution_mask_has_exactly_10_bits_set() {
        for id in 0..1000 {
            let round = default_round(id);
            let mask = round.distribution_mask();
            assert_eq!(
                mask.count_ones(),
                10,
                "Round {id}: expected 10 bits set, got {}",
                mask.count_ones()
            );
        }
    }

    #[test]
    fn test_distribution_mask_only_uses_first_25_bits() {
        for id in 0..1000 {
            let round = default_round(id);
            let mask = round.distribution_mask();
            assert_eq!(
                mask & !((1u32 << 25) - 1),
                0,
                "Round {id}: bits above position 24 should not be set"
            );
        }
    }

    #[test]
    fn test_distribution_mask_is_deterministic() {
        for id in 0..100 {
            let round = default_round(id);
            let mask1 = round.distribution_mask();
            let mask2 = round.distribution_mask();
            assert_eq!(mask1, mask2, "Round {id}: mask should be deterministic");
        }
    }

    #[test]
    fn test_distribution_mask_values_are_randomized() {
        let mut masks = std::collections::HashSet::new();
        for id in 0..100 {
            let round = default_round(id);
            masks.insert(round.distribution_mask());
        }
        assert!(
            masks.len() > 50,
            "Expected diverse mask values across rounds, only got {} unique values out of 100",
            masks.len()
        );
    }
}
