pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod loaders;
pub mod state;

pub(crate) use utils;

use solana_program::declare_id;

declare_id!("mineRHF5r6S7HyD9SppBfVMXMavDkJsxwGesEvxZr2A");
