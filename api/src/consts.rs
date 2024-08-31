use array_const_fn_init::array_const_fn_init;
use const_crypto::ed25519;
use solana_program::{pubkey, pubkey::Pubkey};

/// The authority allowed to initialize the program.
pub const INITIALIZER_ADDRESS: Pubkey = pubkey!("FJka1yJHn1SWux2X1o8VqHC8uaAWGv6CbNQvPWLJQufq");

/// The base reward rate to intialize the program with.
pub const INITIAL_BASE_COAL_REWARD_RATE: u64 = BASE_COAL_REWARD_RATE_MIN_THRESHOLD;
pub const INITIAL_BASE_WOOD_REWARD_RATE: u64 = BASE_WOOD_REWARD_RATE_MIN_THRESHOLD;

/// The minimum allowed base reward rate, at which point the min difficulty should be increased
pub const BASE_COAL_REWARD_RATE_MIN_THRESHOLD: u64 = 2u64.pow(5) * COAL_EXTRACTION_MULTIPLIER;

/// The maximum allowed base reward rate, at which point the min difficulty should be decreased.
pub const BASE_COAL_REWARD_RATE_MAX_THRESHOLD: u64 = 2u64.pow(8) * COAL_EXTRACTION_MULTIPLIER;

/// The minimum allowed base reward rate, at which point the min difficulty should be increased
pub const BASE_WOOD_REWARD_RATE_MIN_THRESHOLD: u64 = 2u64.pow(5) * WOOD_EXTRACTION_MULTIPLIER;

/// The maximum allowed base reward rate, at which point the min difficulty should be decreased.
pub const BASE_WOOD_REWARD_RATE_MAX_THRESHOLD: u64 = 2u64.pow(8) * WOOD_EXTRACTION_MULTIPLIER;


// When total hash power is above this threshold, wood emissions grows at a faster rate
// When total hash power is below this threshold, wood emissions decline a rate
pub const WOOD_GROWTH_THESHOLD: u64 = 500; 

/// The spam/liveness tolerance in seconds.
pub const TOLERANCE: i64 = 5;

/// The minimum difficulty to initialize the program with.
pub const INITIAL_MIN_DIFFICULTY: u32 = 1;

/// The decimal precision of the COAL token.
/// There are 100 billion indivisible units per COAL (called "grains").
pub const TOKEN_DECIMALS: u8 = 11;

/// One COAL token, denominated in indivisible units.
pub const ONE_COAL: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// One WOOD token, denominated in indivisible units.
pub const ONE_WOOD: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// The duration of one minute, in seconds.
pub const ONE_MINUTE: i64 = 60;

/// The number of minutes in a program epoch.
pub const COAL_EPOCH_MINUTES: i64 = 2;
pub const WOOD_EPOCH_MINUTES: i64 = 10;

/// The duration of a program epoch, in seconds.
pub const COAL_EPOCH_DURATION: i64 = ONE_MINUTE * COAL_EPOCH_MINUTES;
pub const WOOD_EPOCH_DURATION: i64 = ONE_MINUTE * WOOD_EPOCH_MINUTES;
/// The maximum token supply (21 million).
pub const MAX_COAL_SUPPLY: u64 = ONE_COAL * 21_000_000;

/// The multiplier for the target quantity of COAL to be mined per epoch.
pub const COAL_EXTRACTION_MULTIPLIER: u64 = 1000;
pub const WOOD_EXTRACTION_MULTIPLIER: u64 = 10;

/// The target quantity of COAL to be mined per epoch.
pub const TARGET_COAL_EPOCH_REWARDS: u64 = ONE_COAL * COAL_EXTRACTION_MULTIPLIER * COAL_EPOCH_MINUTES as u64;

/// The propogation multiplier for WOOD emissions.
pub const WOOD_PROPOGATION_MULTIPLIER: f64 = 1.05;

/// The maximum quantity of COAL that can be mined per epoch.
/// Inflation rate ≈ 1000 COAL / min (min 0, max 8)
pub const MAX_COAL_EPOCH_REWARDS: u64 = TARGET_COAL_EPOCH_REWARDS * BUS_COUNT as u64;

/// The quantity of COAL each bus is allowed to issue per epoch.
pub const BUS_COAL_EPOCH_REWARDS: u64 = MAX_COAL_EPOCH_REWARDS / BUS_COUNT as u64;

/// The number of bus accounts, for parallelizing mine operations.
pub const BUS_COUNT: usize = 8;

/// The smoothing factor for reward rate changes. The reward rate cannot change by mCOAL or less
/// than a factor of this constant from one epoch to the next.
pub const SMOOTHING_FACTOR: u64 = 2;

// Assert MAX_EPOCH_REWARDS is evenly divisible by BUS_COUNT.
static_assertions::const_assert!(
    (MAX_COAL_EPOCH_REWARDS / BUS_COUNT as u64) * BUS_COUNT as u64 == MAX_COAL_EPOCH_REWARDS
);

/// The seed of the bus account PDA.
pub const COAL_BUS: &[u8] = b"bus";
pub const WOOD_BUS: &[u8] = b"wood_bus";

/// The seed of the config account PDA.
pub const COAL_CONFIG: &[u8] = b"config";
pub const WOOD_CONFIG: &[u8] = b"wood_config";

/// The seed of the metadata account PDA.
pub const METADATA: &[u8] = b"metadata";

/// The seed of the mint account PDA.
pub const COAL_MINT: &[u8] = b"mint";
pub const WOOD_MINT: &[u8] = b"wood_mint";

/// The seed of proof account PDAs.
pub const COAL_PROOF: &[u8] = b"proof";
pub const WOOD_PROOF: &[u8] = b"wood_proof";
/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// Noise for deriving the mint pda
pub const MINT_NOISE: [u8; 16] = [
    89, 157, 88, 232, 243, 249, 197, 132, 199, 49, 19, 234, 91, 94, 150, 41,
];

/// The name for token metadata.
pub const COAL_METADATA_NAME: &str = "coal";
pub const WOOD_METADATA_NAME: &str = "wood";

/// The ticker symbol for token metadata.
pub const COAL_METADATA_SYMBOL: &str = "COAL";
pub const WOOD_METADATA_SYMBOL: &str = "WOOD";

/// The uri for token metdata.
pub const COAL_METADATA_URI: &str = "https://coal.digital/metadata.json";
pub const WOOD_METADATA_URI: &str = "https://coal.digital/metadata.wood.json";

/// Program id for const pda derivations
const PROGRAM_ID: [u8; 32] = unsafe { *(&crate::id() as *const Pubkey as *const [u8; 32]) };

/// ORE program id 
pub const ORE_PROGRAM_ID: Pubkey = pubkey!("oreV2ZymfyeXgNgBdqMkumTqqAprVqgBWQfoYkrtKWQ");
pub const ORE_PROGRAM_ID_BYTES: [u8; 32] = unsafe { *(&ORE_PROGRAM_ID as *const Pubkey as *const [u8; 32]) };

/// The addresses of the bus accounts.
pub const COAL_BUS_ADDRESSES: [Pubkey; BUS_COUNT] = array_const_fn_init![const_coal_bus_address; 8];
pub const WOOD_BUS_ADDRESSES: [Pubkey; BUS_COUNT] = array_const_fn_init![const_wood_bus_address; 8];

/// Function to derive const bus addresses.
const fn const_coal_bus_address(i: usize) -> Pubkey {
    Pubkey::new_from_array(ed25519::derive_program_address(&[COAL_BUS, &[i as u8]], &PROGRAM_ID).0)
}

const fn const_wood_bus_address(i: usize) -> Pubkey {
    Pubkey::new_from_array(ed25519::derive_program_address(&[WOOD_BUS, &[i as u8]], &PROGRAM_ID).0)
}

/// The address of the config account.
pub const COAL_CONFIG_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[COAL_CONFIG], &PROGRAM_ID).0);
pub const WOOD_CONFIG_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[WOOD_CONFIG], &PROGRAM_ID).0);

/// The address of the mint metadata account.
pub const COAL_METADATA_ADDRESS: Pubkey = Pubkey::new_from_array(
    ed25519::derive_program_address(
        &[
            METADATA,
            unsafe { &*(&mpl_token_metadata::ID as *const Pubkey as *const [u8; 32]) },
            unsafe { &*(&COAL_MINT_ADDRESS as *const Pubkey as *const [u8; 32]) },
        ],
        unsafe { &*(&mpl_token_metadata::ID as *const Pubkey as *const [u8; 32]) },
    )
    .0,
);
pub const WOOD_METADATA_ADDRESS: Pubkey = Pubkey::new_from_array(
    ed25519::derive_program_address(
        &[
            METADATA,
            unsafe { &*(&mpl_token_metadata::ID as *const Pubkey as *const [u8; 32]) },
            unsafe { &*(&WOOD_MINT_ADDRESS as *const Pubkey as *const [u8; 32]) },
        ],
        unsafe { &*(&mpl_token_metadata::ID as *const Pubkey as *const [u8; 32]) },
    )
    .0,
);

/// The address of the mint account.
pub const COAL_MINT_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[COAL_MINT, &MINT_NOISE], &PROGRAM_ID).0);

/// The address of the mint account.
pub const WOOD_MINT_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[WOOD_MINT, &MINT_NOISE], &PROGRAM_ID).0);

/// The address of the treasury account.
pub const TREASURY_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[TREASURY], &PROGRAM_ID).0);

/// The bump of the treasury account, for cpis.
pub const TREASURY_BUMP: u8 = ed25519::derive_program_address(&[TREASURY], &PROGRAM_ID).1;

/// The address of the treasury token account.
pub const TREASURY_TOKENS_ADDRESS: Pubkey = Pubkey::new_from_array(
    ed25519::derive_program_address(
        &[
            unsafe { &*(&TREASURY_ADDRESS as *const Pubkey as *const [u8; 32]) },
            unsafe { &*(&spl_token::id() as *const Pubkey as *const [u8; 32]) },
            unsafe { &*(&COAL_MINT_ADDRESS as *const Pubkey as *const [u8; 32]) },
        ],
        unsafe { &*(&spl_associated_token_account::id() as *const Pubkey as *const [u8; 32]) },
    )
    .0,
);

/// The address of the CU-optimized Solana noop program.
pub const NOOP_PROGRAM_ID: Pubkey = pubkey!("noop8ytexvkpCuqbf6FB89BSuNemHtPRqaNC31GWivW");
