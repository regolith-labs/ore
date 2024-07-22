pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod loaders;
pub mod state;

pub(crate) use ore_utils as utils;

use solana_program::declare_id;

declare_id!("orewfiPagLonm3yZUectXuwSP8wyDhUcb66Gg9GX9q9");
