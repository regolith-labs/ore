use steel::*;

use super::OreAccount;

// TODO Config stuff

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Config {}

account!(OreAccount, Config);
