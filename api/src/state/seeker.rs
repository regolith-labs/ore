use steel::*;

use super::OreAccount;

/// Seeker tracks which Seeker genesis tokens have been claimed.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Seeker {
    // The mint address of a Seeker genesis token.
    pub mint: Pubkey,
}

account!(OreAccount, Seeker);
