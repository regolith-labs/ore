use algonaut_core::Address;

/// The authority allowed to initialize the program.
pub const ADMIN_ADDRESS: &str = "HBUh9g46wk2X89CvaNN15UmsznP59rh6od1h8JwYAopk";

/// Helper to get admin address
pub fn admin_address() -> Address {
    ADMIN_ADDRESS.parse().expect("Invalid admin address")
}

/// The decimal precision of the fPOW token.
/// There are 100 billion indivisible units per fPOW (called "grams").
pub const TOKEN_DECIMALS: u8 = 11;

/// One fPOW token, denominated in indivisible units.
pub const ONE_FPOW: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// The duration of one minute, in seconds.
pub const ONE_MINUTE: i64 = 60;

/// The duration of one hour, in seconds.
pub const ONE_HOUR: i64 = 60 * ONE_MINUTE;

/// The duration of one day, in seconds.
pub const ONE_DAY: i64 = 24 * ONE_HOUR;

/// The number of seconds for when the winning square expires.
pub const ONE_WEEK: i64 = 7 * ONE_DAY;

/// Algorand block time is approximately 3.3 seconds
/// One minute is approximately 18 rounds
pub const ONE_MINUTE_ROUNDS: u64 = 18;

/// The number of rounds in one hour.
pub const ONE_HOUR_ROUNDS: u64 = 60 * ONE_MINUTE_ROUNDS;

/// The number of rounds in 12 hours.
pub const TWELVE_HOURS_ROUNDS: u64 = 12 * ONE_HOUR_ROUNDS;

/// The number of rounds in one day.
pub const ONE_DAY_ROUNDS: u64 = 24 * ONE_HOUR_ROUNDS;

/// The number of rounds in one week.
pub const ONE_WEEK_ROUNDS: u64 = 7 * ONE_DAY_ROUNDS;

/// The number of rounds for breather between rounds.
pub const INTERMISSION_ROUNDS: u64 = 5;

/// The maximum token supply (5 million).
pub const MAX_SUPPLY: u64 = ONE_FPOW * 5_000_000;

/// The seed of the automation account box.
pub const AUTOMATION: &[u8] = b"automation";

/// The seed of the board account box.
pub const BOARD: &[u8] = b"board";

/// The seed of the config account box.
pub const CONFIG: &[u8] = b"config";

/// The seed of the miner account box.
pub const MINER: &[u8] = b"miner";

/// The seed of the seeker account box.
pub const SEEKER: &[u8] = b"seeker";

/// The seed of the square account box.
pub const SQUARE: &[u8] = b"square";

/// The seed of the stake account box.
pub const STAKE: &[u8] = b"stake";

/// The seed of the round account box.
pub const ROUND: &[u8] = b"round";

/// The seed of the treasury account box.
pub const TREASURY: &[u8] = b"treasury";

/// The fPOW ASA (Algorand Standard Asset) ID
/// To be set after asset creation
pub const FPOW_ASA_ID: u64 = 0;

/// The address to indicate fPOW rewards are split between all miners.
pub const SPLIT_ADDRESS: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

/// Denominator for fee calculations.
pub const DENOMINATOR_BPS: u64 = 10_000;

/// The fee paid to bots if they checkpoint a user (in microalgos).
pub const CHECKPOINT_FEE: u64 = 10_000; // 0.01 ALGO

/// Amount paid to bots per transaction for auto-compounding staking yield, in microalgos.
pub const COMPOUND_FEE_PER_TRANSACTION: u64 = 7_000;

/// The fee paid to the admin for each transaction (basis points).
pub const ADMIN_FEE: u64 = 100; // 1%

/// The address to receive the admin fee.
pub const ADMIN_FEE_COLLECTOR: &str = "DyB4Kv6V613gp2LWQTq1dwDYHGKuUEoDHnCouGUtxFiX";

/// The address which can call the bury and wrap instructions.
pub const BURY_AUTHORITY: &str = "HNWhK5f8RMWBqcA7mXJPaxdTPGrha3rrqUrri7HSKb3T";

/// Minimum balance requirement for Algorand accounts (in microalgos)
pub const MIN_BALANCE: u64 = 100_000; // 0.1 ALGO

/// Transaction fee on Algorand (in microalgos)
pub const TXN_FEE: u64 = 1_000; // 0.001 ALGO
