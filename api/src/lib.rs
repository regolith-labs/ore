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

use steel::*;

declare_id!("34kSqXoUpB8bSLJ4nkWJ3R3nD1bx1KwvKi1LPxk4FcCk");
