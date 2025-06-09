use steel::*;

use crate::state::permit_pda;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Permit {
    /// The amount of ORE this miner has mined.
    pub amount: u64,

    /// The authority of this miner account.
    pub authority: Pubkey,

    /// The ID of the last block this miner mined in.
    pub block_id: u64,
}

impl Permit {
    pub fn pda(&self) -> (Pubkey, u8) {
        permit_pda(self.authority, self.block_id)
    }
}

account!(OreAccount, Permit);
