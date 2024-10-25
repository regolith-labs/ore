use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum OreInstruction {
    // User
    Claim = 0,
    Close = 1,
    Mine = 2,
    Open = 3,
    Reset = 4,
    #[deprecated(since = "2.4.0", note = "Please stake with the boost program")]
    Stake = 5,
    Update = 6,
    Upgrade = 7,

    // Admin
    Initialize = 100,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Claim {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Close {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Mine {
    pub digest: [u8; 16],
    pub nonce: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Open {
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bump: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Reset {}

#[deprecated(since = "2.4.0", note = "Please stake with the boost program")]
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Stake {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Update {}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Upgrade {
    pub amount: [u8; 8],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Initialize {
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_0_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_1_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_2_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_3_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_4_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_5_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_6_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub bus_7_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub config_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub metadata_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub mint_bump: u8,
    #[deprecated(since = "2.5.0", note = "Bump no longer used")]
    pub treasury_bump: u8,
}

instruction!(OreInstruction, Claim);
instruction!(OreInstruction, Close);
instruction!(OreInstruction, Mine);
instruction!(OreInstruction, Open);
instruction!(OreInstruction, Reset);
instruction!(OreInstruction, Stake);
instruction!(OreInstruction, Update);
instruction!(OreInstruction, Upgrade);
instruction!(OreInstruction, Initialize);
