use std::{fmt, mem::transmute};

use bytemuck::{Pod, Zeroable};
use solana_program::keccak::{Hash as KeccakHash, HASH_BYTES};

use crate::impl_to_bytes;

/// Hash is an equivalent type to solana_program::keccak::Hash which supports bytemuck serialization.
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

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}

impl Hash {
    pub fn difficulty(&self) -> u32 {
        let mut count = 0;
        for &byte in &self.0 {
            let lz = byte.leading_zeros();
            count += lz;
            if lz < 8 {
                break;
            }
        }
        count
    }
}

impl_to_bytes!(Hash);
