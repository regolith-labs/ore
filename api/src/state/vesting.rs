use steel::*;

use super::OreAccount;

/// Vesting accounts track a miner's current vesting schedule.
/// Miners are allowed to claim 1% of their earnings every 24 hours.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Vesting {
    /// The proof assocaited with this vesting schedule.
    pub proof: Pubkey,

    /// The amount of tokens that have been claimed in the current window.
    pub window_claim_amount: u64,

    /// The high water mark of the proof balance.
    pub window_proof_balance: u64,

    /// The start of the vesting window.
    pub window_start_at: i64,
}

account!(OreAccount, Vesting);
