use std::mem::transmute;

use bytemuck::{Pod, Zeroable};
use solana_program::keccak::{Hash as KeccakHash, HASH_BYTES};

use crate::{impl_account_from_bytes, impl_to_bytes};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Hash(pub [u8; HASH_BYTES]);

impl From<KeccakHash> for Hash {
    #[inline(always)]
    fn from(value: KeccakHash) -> Self {
        unsafe { transmute(value) }
    }
}

impl From<Hash> for KeccakHash {
    #[inline(always)]
    fn from(value: Hash) -> Self {
        unsafe { transmute(value) }
    }
}

impl_to_bytes!(Hash);
impl_account_from_bytes!(Hash);
