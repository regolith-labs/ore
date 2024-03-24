use solana_program::{keccak::Hash, pubkey, pubkey::Pubkey};

/// The unix timestamp after which mining is allowed.
pub const START_AT: i64 = 1712070900;

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

/// Noise for deriving the mint pda
pub const MINT_NOISE: [u8; 16] = [
    166, 199, 85, 221, 225, 119, 21, 185, 160, 82, 242, 237, 194, 84, 250, 252,
];

/// The addresses of the bus accounts.
pub const BUS_ADDRESSES: [Pubkey; BUS_COUNT] = [
    pubkey!("9ShaCzHhQNvH8PLfGyrJbB8MeKHrDnuPMLnUDLJ2yMvz"),
    pubkey!("4Cq8685h9GwsaD5ppPsrtfcsk3fum8f9UP4SPpKSbj2B"),
    pubkey!("8L1vdGdvU3cPj9tsjJrKVUoBeXYvAzJYhExjTYHZT7h7"),
    pubkey!("JBdVURCrUiHp4kr7srYtXbB7B4CwurUt1Bfxrxw6EoRY"),
    pubkey!("DkmVBWJ4CLKb3pPHoSwYC2wRZXKKXLD2Ued5cGNpkWmr"),
    pubkey!("9uLpj2ZCMqN6Yo1vV6yTkP6dDiTTXmeM5K3915q5CHyh"),
    pubkey!("EpcfjBs8eQ4unSMdowxyTE8K3vVJ3XUnEr5BEWvSX7RB"),
    pubkey!("Ay5N9vKS2Tyo2M9u9TFt59N1XbxdW93C7UrFZW3h8sMC"),
];

/// The address of the Ore mint metadata account.
pub const METADATA_ADDRESS: Pubkey = pubkey!("2nXZSxfjELuRatcoY64yHdFLZFi3mtesxobHmsoU3Dag");

/// The address of the Ore mint account.
pub const MINT_ADDRESS: Pubkey = pubkey!("oreoN2tQbHXVaZsr3pf66A48miqcBXCDJozganhEJgz");

/// The address of the treasury account.
pub const TREASURY_ADDRESS: Pubkey = pubkey!("FTap9fv2GPpWGqrLj3o4c9nHH7p36ih7NbSWHnrkQYqa");
