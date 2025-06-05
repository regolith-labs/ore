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

/// The seed of the commit account PDA.
pub const COMMIT: &[u8] = b"commit";

/// The seed of the config account PDA.
pub const CONFIG: &[u8] = b"config";

/// The seed of the market account PDA.
pub const MARKET: &[u8] = b"market";

/// The seed of the miner account PDA.
pub const MINER: &[u8] = b"miner";

/// The seed of the receipt account PDA.
pub const RECEIPT: &[u8] = b"receipt";

/// The seed of the stake account PDA.
pub const STAKE: &[u8] = b"stake";

/// The seed of the metadata account PDA.
pub const METADATA: &[u8] = b"metadata";

/// The seed of the mint account PDA.
pub const MINT: &[u8] = b"mint";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// Noise for deriving the mint pda
pub const MINT_NOISE: [u8; 16] = [
    89, 157, 88, 232, 243, 249, 197, 132, 199, 49, 19, 234, 91, 94, 150, 41,
];

/// The name for token metadata.
pub const METADATA_NAME: &str = "ORE";

/// The ticker symbol for token metadata.
pub const METADATA_SYMBOL: &str = "ORE";

/// The uri for token metdata.
pub const METADATA_URI: &str = "https://ore.supply/assets/metadata.json";

/// Program id for const pda derivations
const PROGRAM_ID: [u8; 32] = unsafe { *(&crate::id() as *const Pubkey as *const [u8; 32]) };

/// The address of the config account.
pub const CONFIG_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[CONFIG], &PROGRAM_ID).0);

/// The address of the mint metadata account.
pub const METADATA_ADDRESS: Pubkey = Pubkey::new_from_array(
    ed25519::derive_program_address(
        &[
            METADATA,
            unsafe { &*(&mpl_token_metadata::ID as *const Pubkey as *const [u8; 32]) },
            unsafe { &*(&MINT_ADDRESS as *const Pubkey as *const [u8; 32]) },
        ],
        unsafe { &*(&mpl_token_metadata::ID as *const Pubkey as *const [u8; 32]) },
    )
    .0,
);

/// The address of the mint account.
pub const MINT_ADDRESS: Pubkey =
    Pubkey::new_from_array(ed25519::derive_program_address(&[MINT, &MINT_NOISE], &PROGRAM_ID).0);

/// The bump of the mint account.
pub const MINT_BUMP: u8 = ed25519::derive_program_address(&[MINT, &MINT_NOISE], &PROGRAM_ID).1;

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
            unsafe { &*(&MINT_ADDRESS as *const Pubkey as *const [u8; 32]) },
        ],
        unsafe { &*(&spl_associated_token_account::id() as *const Pubkey as *const [u8; 32]) },
    )
    .0,
);

/// Denominator for protocol fee calculations.
pub const FEE_RATE_BPS: u64 = 100;

/// Denominator for fee calculations.
pub const DENOMINATOR_BPS: u64 = 10_000;

/// Slot window size, used for sandwich resistance.
pub const SLOT_WINDOW: u64 = 4;
