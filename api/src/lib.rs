pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod loaders;
pub mod state;

pub(crate) use ore_utils as utils;

use solana_program::declare_id;

// declare_id!("mineRHF5r6S7HyD9SppBfVMXMavDkJsxwGesEvxZr2A");
declare_id!("5vBA1LwxxiobFsVHZeHgWS5mdUTjRDjo7e7xSFk25LVr");
