use steel::*;

use crate::state::block_pda;

use super::OreAccount;

// What could be variable?
// - Payout style (winner take all / difficulty / both)
// - Payout skew (larger / neutral / smaller)
// - Jackpot possiblity (yes / no)
// - Known / unknown

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Block {
    /// The block number.
    pub id: u64,

    /// The minimum difficulty required for payout.
    // pub min_difficulty: u64,

    /// The reward rate per satisfying hash.
    // pub reward_rate: u64,
    pub reward: RewardConfig,

    /// The hash of the starting slot.
    pub slot_hash: [u8; 32],

    /// The starting slot of the block.
    pub start_slot: u64,

    /// The total number of hashes submitted to the block.
    pub total_hashes: u64,

    /// The total amount of rewards paid out to miners.
    pub total_rewards: u64,

    /// The total number of hashes that resulted in a payout.
    pub winning_hashes: u64,
}

/// Configuration specifying how rewards are paid out.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct RewardConfig {
    /// The reward paid to the submitter of the best hash.
    pub best_hash_reward: u64,

    /// The authority of the miner who submitted the best hash.
    pub best_hash_authority: Pubkey,

    /// The best hash.
    pub best_hash: [u8; 32],

    /// The reward rate paid to hashes satisfying the difficulty threshold.
    pub difficulty_reward: u64,

    /// The minimum difficulty required for payout.
    pub difficulty_threshold: u64,
}

impl Block {
    pub fn pda(&self) -> (Pubkey, u8) {
        block_pda(self.id)
    }
}

account!(OreAccount, Block);
