use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;

use crate::{
    impl_account_from_bytes, impl_to_bytes,
    state::Hash,
    utils::{AccountDiscriminator, Discriminator},
};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {
    /// The bump of the treasury account PDA.
    pub bump: u64,

    /// The admin authority with permission to update the difficulty.
    pub admin: Pubkey,

    /// The hash difficulty.
    pub difficulty: Hash,

    /// The timestamp of the start of the current epoch.
    pub epoch_start_at: i64,

    /// The reward rate to payout to miners for submiting valid hashes.
    pub reward_rate: u64,

    /// The total lifetime claimed rewards.
    pub total_claimed_rewards: u64,
}

impl Discriminator for Treasury {
    fn discriminator() -> AccountDiscriminator {
        AccountDiscriminator::Treasury
    }
}

impl_to_bytes!(Treasury);
impl_account_from_bytes!(Treasury);
