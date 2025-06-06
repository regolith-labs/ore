use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Miner {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The ID of the last block this miner mined in.
    pub block_id: u64,

    /// The hash of the last block this miner mined in.
    pub hash: [u8; 32],

    /// The total number of hashes this miner has submitted.
    pub total_hashes: u64,

    /// The amount of ORE this miner has mined.
    pub total_rewards: u64,
}

account!(OreAccount, Miner);
