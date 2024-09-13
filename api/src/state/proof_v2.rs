use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;


use crate::utils::{impl_account_from_bytes, impl_to_bytes, Discriminator};


use super::AccountDiscriminator;


#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ProofV2 {
    /// The resource mint this proof is for.
    pub resource: Pubkey,

    /// The signer authorized to use this proof.
    pub authority: Pubkey,

    /// The quantity of tokens this miner has staked or earned.
    pub balance: u64,

    /// The current mining challenge.
    pub challenge: [u8; 32],

    /// The last hash the miner provided.
    pub last_hash: [u8; 32],

    /// The last time this account provided a hash.
    pub last_hash_at: i64,

    /// The last time stake was deposited into this account.
    pub last_stake_at: i64,

    /// The keypair which has permission to submit hashes for mining.
    pub miner: Pubkey,

    /// The total lifetime hashes provided by this miner.
    pub total_hashes: u64,

    /// The total lifetime rewards distributed to this miner.
    pub total_rewards: u64,

    /// The tool equipped by the miner.
    pub equipped_tool: Pubkey,
}

impl Discriminator for ProofV2 {
    fn discriminator() -> u8 {
        AccountDiscriminator::ProofV2.into()
    }
}

impl_to_bytes!(ProofV2);
impl_account_from_bytes!(ProofV2);
