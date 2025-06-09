use steel::*;

use crate::state::permit_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Permit {
    /// The amount of hash tokens this miner has committed to the block.
    pub amount: u64,

    /// The authority of the miner account.
    pub authority: Pubkey,

    /// The ID of the block this permit is for.
    pub block_id: u64,
}

impl Permit {
    pub fn pda(&self) -> (Pubkey, u8) {
        permit_pda(self.authority, self.block_id)
    }
}

account!(OreAccount, Permit);
