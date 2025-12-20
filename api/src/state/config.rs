use serde::{Deserialize, Serialize};

use crate::state::config_box_name;

/// Configuration state for the fPOW application
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The address that can update the config (Algorand address bytes).
    pub admin: [u8; 32],

    /// Buffer a (placeholder)
    pub buffer_a: [u8; 32],

    /// Buffer b (placeholder)
    pub buffer_b: [u8; 32],

    /// Buffer c (placeholder)
    pub buffer_c: [u8; 32],

    /// Buffer d (placeholder)
    pub buffer_d: [u8; 32],

    /// Buffer e (placeholder)
    pub buffer_e: [u8; 8],
}

impl Config {
    pub fn box_name() -> Vec<u8> {
        config_box_name()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            admin: [0u8; 32],
            buffer_a: [0u8; 32],
            buffer_b: [0u8; 32],
            buffer_c: [0u8; 32],
            buffer_d: [0u8; 32],
            buffer_e: [0u8; 8],
        }
    }
}
