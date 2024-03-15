use solana_program::{keccak::Hash, pubkey, pubkey::Pubkey};

/// The unix timestamp after which mining is allowed.
pub const START_AT: i64 = 0;

/// The addresses of the bus accounts.
pub const BUS_ADDRESSES: [Pubkey; BUS_COUNT] = [
    pubkey!("BipsVd7bXrsWTD7MSR7vEE6wCi812xree9MdMQHNpGve"),
    pubkey!("4UywqKWK1hFk2FVn2i8dHGRL28wTmwv2pveBV4QhApsy"),
    pubkey!("6q971fxeEjwQWXL3PTEWBpSAs55HvKkb4yVeP85qrycY"),
    pubkey!("BWa6yzTg3Mrs6tH2rstKQrobSVEBkfQUzE4iZUJz54AP"),
    pubkey!("AXdpmjuVJYk7re8TcDX33ZoLyYpbBopesRWaNYnDHe8L"),
    pubkey!("FXikoBAemf48cZswLnnUtXZhD7Pr6knYPNq6cjb76ubh"),
    pubkey!("9BqvaWJ9bmkPYNqaZPaHoonAR5bh1dHRymZV4nrae3sC"),
    pubkey!("8T4ngp27XKn3fAJnHgZ9pDXwzMbHnxhCPuvBWMa5m3ro"),
];

/// The address of the Ore mint metadata account.
pub const METADATA_ADDRESS: Pubkey = pubkey!("wyJ7XtZQDox5XoPG3uw7u7XnpNXaqTDeAwKpuJqHLi4");

/// The address of the Ore mint account.
pub const MINT_ADDRESS: Pubkey = pubkey!("oreoBXz6dRgETAVLre1Umgp6Hs4UdLRwJiYj5FkfzYh");

/// The address of the treasury account.
pub const TREASURY_ADDRESS: Pubkey = pubkey!("CHTwJ2GLmz9KDEpPuLu5iYFF85pSzx4xzJSwm81ojN3Q");

/// The initial reward rate to payout in the first epoch.
pub const INITIAL_REWARD_RATE: u64 = 10u64.pow(3u32);

/// The initial hashing difficulty. The admin authority can update this in the future, if needed.
pub const INITIAL_DIFFICULTY: Hash = Hash::new_from_array([
    0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
]);

/// The decimal precision of the ORE token.
/// Using SI prefixes, the smallest indivisible unit of ORE is a nanoORE.
/// 1 nanoORE = 0.000000001 ORE = one billionth of an ORE
pub const TOKEN_DECIMALS: u8 = 9;

/// One ORE token, denominated in units of nanoORE.
pub const ONE_ORE: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// The duration of an epoch, in units of seconds.
pub const EPOCH_DURATION: i64 = 60;

/// The target quantity of ORE to be mined per epoch, in units of nanoORE.
/// Inflation rate ≈ 1 ORE / epoch (min 0, max 2)
pub const TARGET_EPOCH_REWARDS: u64 = ONE_ORE;

/// The maximum quantity of ORE that can be mined per epoch, in units of nanoORE.
pub const MAX_EPOCH_REWARDS: u64 = ONE_ORE.saturating_mul(2);

/// The quantity of ORE each bus is allowed to issue per epoch.
pub const BUS_EPOCH_REWARDS: u64 = MAX_EPOCH_REWARDS.saturating_div(BUS_COUNT as u64);

/// The number of bus accounts, for parallelizing mine operations.
pub const BUS_COUNT: usize = 8;

/// The smoothing factor for reward rate changes. The reward rate cannot change by more or less
/// than factor of this constant from one epoch to the next.
pub const SMOOTHING_FACTOR: u64 = 2;

// Assert MAX_EPOCH_REWARDS is evenly divisible by BUS_COUNT.
static_assertions::const_assert!(
    (MAX_EPOCH_REWARDS / BUS_COUNT as u64) * BUS_COUNT as u64 == MAX_EPOCH_REWARDS
);

/// Noise for deriving the mint pda
pub const MINT_NOISE: [u8; 16] = [
    64, 193, 214, 243, 206, 254, 96, 138, 148, 27, 250, 15, 126, 55, 231, 93,
];

/// The seed of the bus account PDA.
pub const BUS: &[u8] = b"bus";

/// The seed of the mint account PDA.
pub const METADATA: &[u8] = b"metadata";

/// The seed of the mint account PDA.
pub const MINT: &[u8] = b"mint";

/// The seed of the proof account PDA.
pub const PROOF: &[u8] = b"proof";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// The name for token metadata.
pub const METADATA_NAME: &str = "Ore";

/// The ticker symbol for token metadata.
pub const METADATA_SYMBOL: &str = "ORE";

/// The uri for token metdata.
pub const METADATA_URI: &str = "https://ore.supply/metadata.json";
