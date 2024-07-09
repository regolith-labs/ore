pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod loaders;
pub mod state;

pub(crate) use ore_utils as utils;

use solana_program::declare_id;

// declare_id!("mineRHF5r6S7HyD9SppBfVMXMavDkJsxwGesEvxZr2A");
declare_id!("CWZk6C3fGbpr1UprdCGaLvnW2ogh7okEmeXaS161RxUg");
