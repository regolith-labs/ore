use solana_program::{keccak::Hash, pubkey, pubkey::Pubkey};

// TODO Set this before deployment
/// The unix timestamp after which mining is allowed.
pub const START_AT: i64 = 0;

// SHA2 const stable
/// Bus pubkeys
pub const BUS_ADDRESSES: [Pubkey; BUS_COUNT] = [
    pubkey!("2uwqyH2gKqstgAFCSniirx73X4iQek5ETc2vVJKUiNMg"),
    pubkey!("FRMC6jVczm1cRaEs5EhDsfw7X8vsmSDpf3bJWVkawngu"),
    pubkey!("9nWyycs4GHjnLujPR2sbA1A8K8CkiLc5VzxWUD4hg2uM"),
    pubkey!("Kt7kqD3MyvxLbj4ek9urXUxkDoxaMuQn82K2VdYD1jM"),
    pubkey!("8r9mXYnFQXhwrNfvatGUTxbbNSqxScuCwp4sBTSxDVTJ"),
    pubkey!("D9cEH32k8p9uWc4w5RrStK9rWssU8NuX1Dg5YaUim4wL"),
    pubkey!("H1RKMYADPzd4C1j1RZu51NvRSVktoTYEJyeVy98Kmdyu"),
    pubkey!("3XbdZNbBjjp8qnDJjv1RxaKisyfx6ahznYkSigs6dayy"),
];

/// The mint address of the ORE token.
pub const MINT_ADDRESS: Pubkey = pubkey!("DY4JVebraRXg9BGt4MRU4mvqHGDzmi2Ay1HGjDU5YeNf");

/// Treasury address
pub const TREASURY_ADDRESS: Pubkey = pubkey!("67PLJej6iZm915WbEu6NLeZtRZtnHc5nSVQvkHRZyPiC");

/// The initial reward rate to payout in the first epoch.
pub const INITIAL_REWARD_RATE: u64 = 10u64.pow(3u32);

/// The initial hashing difficulty. The admin authority can update this in the future, if needed.
pub const INITIAL_DIFFICULTY: Hash = Hash::new_from_array([
    0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
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

/// The seed of the bus account PDA.
pub const BUS: &[u8] = b"bus";

/// The seed of the mint account PDA.
pub const MINT: &[u8] = b"mint";

/// The seed of the proof account PDA.
pub const PROOF: &[u8] = b"proof";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";
