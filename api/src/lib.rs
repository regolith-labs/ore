pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod sdk;
pub mod state;

pub mod prelude {
    pub use crate::consts::*;
    pub use crate::error::*;
    pub use crate::event::*;
    pub use crate::instruction::*;
    pub use crate::sdk::*;
    pub use crate::state::*;
}

use algonaut_core::Address;

/// The fPOW application ID on Algorand
pub const APP_ID: u64 = 0; // To be set after deployment

/// The fPOW program address (application account)
pub fn program_address() -> Address {
    algonaut_core::to_app_address(APP_ID)
}

/// The ID function for compatibility
pub fn id() -> Address {
    program_address()
}
