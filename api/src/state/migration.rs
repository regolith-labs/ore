use serde::{Deserialize, Serialize};
use steel::*;

use super::OreAccount;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable, Serialize, Deserialize)]
pub struct Migration {}

account!(OreAccount, Migration);
