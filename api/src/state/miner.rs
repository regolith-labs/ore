use steel::*;

use crate::state::miner_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Miner {
    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The miner's committed square in the current round round.
    pub commits: [u64; 25],

    /// The amount of ORE this miner can claim.
    pub rewards: u64,

    /// The ID of the round this miner last played in.
    pub round_id: u64,

    /// The total amount of ORE this miner has mined across all blocks.
    pub total_rewards: u64,
}

impl Miner {
    pub fn pda(&self) -> (Pubkey, u8) {
        miner_pda(self.authority)
    }
}

account!(OreAccount, Miner);
