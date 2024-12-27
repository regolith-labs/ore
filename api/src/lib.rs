pub mod consts;
pub mod error;
pub mod event;
#[allow(deprecated)]
pub mod instruction;
pub mod loaders;
pub mod sdk;
pub mod state;

pub mod prelude {
    pub use crate::consts::*;
    pub use crate::error::*;
    pub use crate::event::*;
    pub use crate::instruction::*;
    pub use crate::loaders::*;
    pub use crate::sdk::*;
    pub use crate::state::*;
}

use steel::*;

declare_id!("3cGMJbUnPeS6M2xaNkA3iRBDAKxtJ99asei4HgAGMEL1");
