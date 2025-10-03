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

/// The duration of one hour, in seconds.
pub const ONE_HOUR: i64 = 60 * ONE_MINUTE;

/// The duration of one day, in seconds.
pub const ONE_DAY: i64 = 24 * ONE_HOUR;

/// The number of seconds for when the winning square expires.
pub const ONE_WEEK: i64 = 7 * ONE_DAY;

/// The number of slots in one week.
pub const ONE_MINUTE_SLOTS: u64 = 150;

/// The number of slots in one hour.
pub const ONE_HOUR_SLOTS: u64 = 60 * ONE_MINUTE_SLOTS;

/// The number of slots in 12 hours.
pub const TWELVE_HOURS_SLOTS: u64 = 12 * ONE_HOUR_SLOTS;

/// The number of slots in one day.
pub const ONE_DAY_SLOTS: u64 = 24 * ONE_HOUR_SLOTS;

/// The number of slots in one week.
pub const ONE_WEEK_SLOTS: u64 = 7 * ONE_DAY_SLOTS;

/// The number of slots for breather between rounds.
pub const INTERMISSION_SLOTS: u64 = 35;

/// The maximum token supply (5 million).
pub const MAX_SUPPLY: u64 = ONE_ORE * 5_000_000;

/// The seed of the automation account PDA.
pub const AUTOMATION: &[u8] = b"automation";

/// The seed of the board account PDA.
pub const BOARD: &[u8] = b"board";

/// The seed of the config account PDA.
pub const CONFIG: &[u8] = b"config";

/// The seed of the miner account PDA.
pub const MINER: &[u8] = b"miner";

/// The seed of the seeker account PDA.
pub const SEEKER: &[u8] = b"seeker";

/// The seed of the square account PDA.
pub const SQUARE: &[u8] = b"square";

/// The seed of the stake account PDA.
pub const STAKE: &[u8] = b"stake";

/// The seed of the round account PDA.
pub const ROUND: &[u8] = b"round";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// Program id for const pda derivations
const PROGRAM_ID: [u8; 32] = unsafe { *(&crate::id() as *const Pubkey as *const [u8; 32]) };

/// The address of the config account.
pub const CONFIG_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[CONFIG], &PROGRAM_ID).0);

/// The address of the mint account.
pub const MINT_ADDRESS: Pubkey = pubkey!("oreoU2P8bN6jkk3jbaiVxYnG1dCXcYxwhwyK9jSybcp");

/// The address of the sol mint account.
pub const SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

/// The address to indicate ORE rewards are split between all miners.
pub const SPLIT_ADDRESS: Pubkey = pubkey!("SpLiT11111111111111111111111111111111111112");

/// The address of the treasury account.
pub const TREASURY_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[TREASURY], &PROGRAM_ID).0);

/// The address of the treasury account.
pub const TREASURY_BUMP: u8 = ed25519::derive_program_address(&[TREASURY], &PROGRAM_ID).1;

/// Denominator for fee calculations.
pub const DENOMINATOR_BPS: u64 = 10_000;

/// The address of the boost reserve token account.
pub const BOOST_RESERVE_TOKEN: Pubkey = pubkey!("Gce36ZUsBDJsoLrfCBxUB5Sfq2DsGunofStvxFx6rBiD");

/// The fee paid to bots if they checkpoint a user.
pub const CHECKPOINT_FEE: u64 = 10_000; // 0.00001 SOL
