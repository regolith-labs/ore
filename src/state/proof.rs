use bytemuck::{Pod, Zeroable};
use shank::ShankAccount;
use solana_program::pubkey::Pubkey;

use crate::{
    impl_account_from_bytes, impl_to_bytes,
    state::Hash,
    utils::{AccountDiscriminator, Discriminator},
};

/// Proof accounts track a miner's current hash, claimable rewards, and lifetime stats.
/// Every miner is allowed one proof account which is required by the program to mine or claim rewards.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, ShankAccount, Zeroable)]
pub struct Proof {
    /// The signer authorized to use this proof.
    pub authority: Pubkey,

    /// The quantity of tokens this miner may claim from the treasury.
    pub balance: u64,

    /// The proof's current hash.
    pub hash: Hash,

    /// The last time this account provided a hash.
    pub last_hash_at: u64,

    // TODO Figure out multiplier representation
    /// The rewards multiplier for this account.
    pub multiplier: u64,

    /// The total lifetime hashes provided by this miner.
    pub total_hashes: u64,

    /// The total lifetime rewards distributed to this miner.
    pub total_rewards: u64,
}

impl Discriminator for Proof {
    fn discriminator() -> AccountDiscriminator {
        AccountDiscriminator::Proof
    }
}

impl_to_bytes!(Proof);
impl_account_from_bytes!(Proof);
