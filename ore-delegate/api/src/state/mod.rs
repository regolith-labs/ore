mod delegate;

pub use delegate::*;

use crate::consts::*;

use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum OreDelegateAccount {
    Delegate = 100,
}

pub fn delegate_pda(authority: Pubkey, block_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[DELEGATE, &authority.to_bytes(), &block_id.to_le_bytes()],
        &crate::ID,
    )
}
