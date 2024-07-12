pub mod consts;
pub mod error;
pub mod event;
pub mod instruction;
pub mod loaders;
pub mod state;

pub(crate) use ore_utils as utils;

use solana_program::declare_id;

// declare_id!("oreV2ZymfyeXgNgBdqMkumTqqAprVqgBWQfoYkrtKWQ");
declare_id!("6XpUkLPF2t28er1Mx89ry5QFJ4piDTJZiEFVRRG3HbS7");
