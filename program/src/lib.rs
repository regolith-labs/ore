//! fPOW Program Utilities
//!
//! This crate provides Rust utilities for interacting with the fPOW smart contract
//! on Algorand. The actual smart contract is written in PyTeal and can be found
//! in the `contract/` directory.
//!
//! ## Smart Contract
//!
//! The fPOW smart contract is implemented in PyTeal at `program/contract/fpow.py`.
//! To compile the contract to TEAL:
//!
//! ```bash
//! cd program/contract
//! pip install -r requirements.txt
//! python fpow.py
//! ```
//!
//! This will generate `approval.teal` and `clear.teal` files.

pub use fpow_api::prelude::*;

/// Utility functions for working with the fPOW contract
pub mod utils {
    use sha2::{Sha512_256, Digest};

    /// Compute the method selector for an ABI method signature
    pub fn method_selector(signature: &str) -> [u8; 4] {
        let hash = Sha512_256::digest(signature.as_bytes());
        let mut selector = [0u8; 4];
        selector.copy_from_slice(&hash[0..4]);
        selector
    }

    /// Compute the address of an Algorand application
    pub fn app_address(app_id: u64) -> [u8; 32] {
        let mut data = b"appID".to_vec();
        data.extend_from_slice(&app_id.to_be_bytes());
        let hash = Sha512_256::digest(&data);
        let mut address = [0u8; 32];
        address.copy_from_slice(&hash);
        address
    }

    /// Convert microalgos to ALGO (for display)
    pub fn microalgos_to_algo(microalgos: u64) -> f64 {
        microalgos as f64 / 1_000_000.0
    }

    /// Convert ALGO to microalgos
    pub fn algo_to_microalgos(algo: f64) -> u64 {
        (algo * 1_000_000.0) as u64
    }

    /// Convert fPOW base units to display units
    pub fn fpow_to_display(amount: u64) -> f64 {
        amount as f64 / 100_000_000_000.0 // 11 decimals
    }

    /// Convert display units to fPOW base units
    pub fn display_to_fpow(amount: f64) -> u64 {
        (amount * 100_000_000_000.0) as u64 // 11 decimals
    }
}

#[cfg(test)]
mod tests {
    use super::utils::*;

    #[test]
    fn test_method_selector() {
        // Test known method selector
        let selector = method_selector("claim_algo()void");
        assert_eq!(selector.len(), 4);
    }

    #[test]
    fn test_app_address() {
        let address = app_address(12345);
        assert_eq!(address.len(), 32);
    }

    #[test]
    fn test_microalgos_conversion() {
        assert_eq!(microalgos_to_algo(1_000_000), 1.0);
        assert_eq!(algo_to_microalgos(1.0), 1_000_000);
    }

    #[test]
    fn test_fpow_conversion() {
        let one_fpow = 100_000_000_000u64; // 11 decimals
        assert_eq!(fpow_to_display(one_fpow), 1.0);
        assert_eq!(display_to_fpow(1.0), one_fpow);
    }
}
