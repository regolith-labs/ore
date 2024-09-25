use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use steel::*;

use crate::consts::BUS;

use super::OreAccount;

/// Bus accounts are responsible for distributing mining rewards. There are 8 busses total
/// to minimize write-lock contention and allow Solana to process mine instructions in parallel.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Bus {
    /// The ID of the bus account.
    pub id: u64,

    /// The remaining rewards this bus has left to payout in the current epoch.
    pub rewards: u64,

    /// The rewards this bus would have paid out in the current epoch if there no limit.
    /// This is used to calculate the updated reward rate.
    pub theoretical_rewards: u64,

    /// The largest known stake balance seen by the bus this epoch.
    pub top_balance: u64,
}

/// Fetch the PDA of a bus account.
pub fn bus_pda(id: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[BUS, &[id]], &crate::id())
}

account!(OreAccount, Bus);
