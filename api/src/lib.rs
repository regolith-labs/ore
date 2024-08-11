pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod loaders;
pub mod state;

pub(crate) use coal_utils as utils;

use solana_program::declare_id;

declare_id!("EG67mGGTxMGuPxDLWeccczVecycmpj2SokzpWeBoGVTf");
