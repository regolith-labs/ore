pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod loaders;
pub mod state;

pub(crate) use coal_utils as utils;

use solana_program::declare_id;

declare_id!("6zZWoA4iyo1f7XsS9J6pFRmjWm3EJU55Z3ym6A4LdCis");
