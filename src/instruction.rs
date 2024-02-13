use bytemuck::{Pod, Zeroable};
use num_enum::TryFromPrimitive;
use shank::ShankInstruction;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, ShankInstruction, TryFromPrimitive)]
#[rustfmt::skip]
pub enum OreInstruction {
    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "bus_0", desc = "Ore bus account 0", writable)]
    #[account(3, name = "bus_1", desc = "Ore bus account 1", writable)]
    #[account(4, name = "bus_2", desc = "Ore bus account 2", writable)]
    #[account(5, name = "bus_3", desc = "Ore bus account 3", writable)]
    #[account(6, name = "bus_4", desc = "Ore bus account 4", writable)]
    #[account(7, name = "bus_5", desc = "Ore bus account 5", writable)]
    #[account(8, name = "bus_6", desc = "Ore bus account 6", writable)]
    #[account(9, name = "bus_7", desc = "Ore bus account 7", writable)]
    #[account(10, name = "mint", desc = "Ore token mint account", writable)]
    #[account(11, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(12, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(13, name = "token_program", desc = "SPL token program")]
    Epoch = 0,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "proof", desc = "Ore miner proof account", writable)]
    #[account(3, name = "system_program", desc = "Solana system program")]
    Proof = 1,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "bus", desc = "Ore bus account", writable)]
    #[account(3, name = "proof", desc = "Ore miner proof account", writable)]
    #[account(4, name = "treasury", desc = "Ore treasury account")]
    #[account(5, name = "token_program", desc = "SPL token program")]
    #[account(6, name = "slot_hashes", desc = "Solana slot hashes sysvar")]
    Mine = 2,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "signer", desc = "Signer", signer)]
    #[account(2, name = "beneficiary", desc = "Beneficiary token account", writable)]
    #[account(3, name = "mint", desc = "Ore token mint account")]
    #[account(4, name = "proof", desc = "Ore miner proof account", writable)]
    #[account(5, name = "treasury", desc = "Ore treasury account", writable)]
    #[account(6, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(7, name = "token_program", desc = "SPL token program")]
    Claim = 3,

    #[account(0, name = "ore_program", desc = "Ore program")]
    #[account(1, name = "admin", desc = "Admin signer", signer)]
    #[account(2, name = "bus_0", desc = "Ore bus account 0", writable)]
    #[account(3, name = "bus_1", desc = "Ore bus account 1", writable)]
    #[account(4, name = "bus_2", desc = "Ore bus account 2", writable)]
    #[account(5, name = "bus_3", desc = "Ore bus account 3", writable)]
    #[account(6, name = "bus_4", desc = "Ore bus account 4", writable)]
    #[account(7, name = "bus_5", desc = "Ore bus account 5", writable)]
    #[account(8, name = "bus_6", desc = "Ore bus account 6", writable)]
    #[account(9, name = "bus_7", desc = "Ore bus account 7", writable)]
    #[account(10, name = "mint", desc = "Ore token mint account")]
    #[account(11, name = "treasury", desc = "Ore treasury account")]
    #[account(12, name = "treasury_tokens", desc = "Ore treasury token account", writable)]
    #[account(13, name = "system_program", desc = "Solana system program")]
    #[account(14, name = "token_program", desc = "SPL token program")]
    #[account(15, name = "rent", desc = "Solana rent sysvar")]
    Initialize = 100,
    // #[account(15, name = "associated_token_program", desc = "SPL associated token program")]

    // TODO
    // #[account(0, name = "ore_program", desc = "Ore program")]
    // UpdateAdmin = 102,

    // TODO
    // #[account(0, name = "ore_program", desc = "Ore program")]
    // UpdateDifficulty = 103,
}

impl OreInstruction {
    pub fn to_vec(&self) -> Vec<u8> {
        vec![*self as u8]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct InitializeArgs {
    pub bus_0_bump: u8,
    pub bus_1_bump: u8,
    pub bus_2_bump: u8,
    pub bus_3_bump: u8,
    pub bus_4_bump: u8,
    pub bus_5_bump: u8,
    pub bus_6_bump: u8,
    pub bus_7_bump: u8,
    pub mint_bump: u8,
    pub treasury_bump: u8,
}

impl InitializeArgs {
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}
