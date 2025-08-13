use const_crypto::ed25519;
use solana_program::{pubkey, pubkey::Pubkey};

/// The authority allowed to initialize the program.
pub const ADMIN_ADDRESS: Pubkey = pubkey!("HBUh9g46wk2X89CvaNN15UmsznP59rh6od1h8JwYAopk");

/// The decimal precision of the ORE token.
/// There are 100 billion indivisible units per ORE (called "grams").
pub const TOKEN_DECIMALS: u8 = 11;

/// One ORE token, denominated in indivisible units.
pub const ONE_ORE: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// The duration of one minute, in seconds.
pub const ONE_MINUTE: i64 = 60;

/// The maximum token supply (5 million).
pub const MAX_SUPPLY: u64 = ONE_ORE * 5_000_000;

/// The seed of the block account PDA.
pub const BLOCK: &[u8] = b"block";

/// The seed of the config account PDA.
pub const CONFIG: &[u8] = b"config";

/// The seed of the market account PDA.
pub const MARKET: &[u8] = b"market";

/// The seed of the miner account PDA.
pub const MINER: &[u8] = b"miner";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// Program id for const pda derivations
const PROGRAM_ID: [u8; 32] = unsafe { *(&crate::id() as *const Pubkey as *const [u8; 32]) };

/// The address of the config account.
pub const CONFIG_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[CONFIG], &PROGRAM_ID).0);

/// The address of the mint account.
pub const MINT_ADDRESS: Pubkey = pubkey!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");

/// The address of the treasury account.
pub const TREASURY_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[TREASURY], &PROGRAM_ID).0);

/// The address of the treasury account.
pub const TREASURY_BUMP: u8 = ed25519::derive_program_address(&[TREASURY], &PROGRAM_ID).1;

/// Swap fee in lamports.
pub const FEE_LAMPORTS: u64 = 100_000; // 0.0001 SOL

/// Denominator for fee calculations.
pub const DENOMINATOR_BPS: u64 = 10_000;

/// Window to submit hashes, in slots.
pub const INITIAL_BLOCK_DURATION: u64 = 1500;

/// Window to submit hashes, in slots.
pub const INITIAL_SNIPER_FEE_DURATION: u64 = 50;

/// Window to submit hashes, in slots.
pub const MINING_WINDOW: u64 = 1500;

/// Slot window size, used for sandwich resistance.
pub const SLOT_WINDOW: u64 = 4;

/// Amount of hash tokens to mint to market.
pub const HASHPOWER_LIQUIDITY: u64 = 1_000_000;

/// The ORE liquidity to seed the markets with.
pub const ORE_LIQUIDITY: u64 = ONE_ORE * 100;

/// The address of the boost reserve token account.
pub const BOOST_RESERVE_TOKEN: Pubkey = pubkey!("5zGJbZ4ZVJ2SNyk34cQoNrmuyJTqqCmTv9crseyFHeww"); // TODO: change this
