use anchor_lang::{
    prelude::*,
    solana_program::{system_program, sysvar},
};
use anchor_spl::token::{self, Mint, MintTo, TokenAccount};
use bnum::types::U256;
use sha256;

declare_id!("CeJShZEAzBLwtcLQvbZc7UT38e4nUTn63Za5UFyYYDTS");

/// The number of hashes per epoch (~24 hours).
pub const EPOCH_HEIGHT: u64 = 8640;

/// The target average duration in seconds between each valid hash.
pub const HASH_TIME: u64 = 10;

/// The seed of the genesis hash.
pub const GENESIS: &str = "42";

/// The seed of the Metadata program derived address.
pub const METADATA: &[u8] = b"metadata";

/// The decimal precision of the Ore token.
pub const TOKEN_DECIMALS: u8 = 8;

/// The radix of U256 string encodings.
pub const RADIX: u32 = 16;

/// The coefficient of the supply function.
pub const SUPPLY_COEFFICIENT: u64 = 48484;

/// The exponent of the supply function.
pub const SUPPLY_EXPONENT: f64 = 0.618;

/// The smoothing factor for difficulty adjustments.
pub const SMOOTHING_FACTOR: U256 = U256::from_digit(4);

// TODO Set this before deployment
/// The time after which mining can happen.
pub const START_AT: i64 = 0;

#[program]
mod ore {
    use super::*;

    /// Initializes the program. Can only be executed once.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let metadata = &mut ctx.accounts.metadata;
        metadata.bump = ctx.bumps.metadata;
        metadata.difficulty = U256::MAX.to_str_radix(RADIX);
        metadata.hash = sha256::digest(GENESIS.to_string());
        metadata.height = 0;
        metadata.mint = ctx.accounts.mint.key();
        Ok(())
    }

    /// Mints new Ore to the beneficiary if a valid hash is provided.
    pub fn mine(ctx: Context<Mine>, hash: String, nonce: u64) -> Result<()> {
        // Validate clock.
        let clock = Clock::get().unwrap();
        require!(clock.unix_timestamp.ge(&START_AT), ProgramError::NotStarted);

        // Log request.
        let metadata = &mut ctx.accounts.metadata;
        msg!("Difficulty: {}", metadata.difficulty);
        msg!("Hash: {}", hash);
        msg!("Nonce: {}", nonce);

        // Validate hash.
        let difficulty = U256::parse_str_radix(&metadata.difficulty, RADIX);
        validate_hash(
            metadata.hash.clone(),
            hash.clone(),
            ctx.accounts.signer.key(),
            nonce,
            difficulty,
        )?;
        msg!("Hash is valid");

        // Update metadata.
        metadata.hash = hash.clone();
        metadata.height = metadata.height.checked_add(1).unwrap();
        if metadata.height.eq(&1) {
            metadata.epoch_start_at = clock.unix_timestamp;
        }

        // Update difficulty, if new epoch.
        if metadata.height % (EPOCH_HEIGHT as u128) == 0 {
            metadata.difficulty =
                calculate_new_difficulty(metadata.epoch_start_at, clock.unix_timestamp, difficulty)
                    .expect("Failed to calculate new difficulty")
                    .to_str_radix(RADIX);
            metadata.epoch_start_at = clock.unix_timestamp;
        }

        // Mint reward to beneficiary.
        let supply = calculate_supply(metadata.height);
        let reward = supply
            .checked_sub(ctx.accounts.mint.supply)
            .expect("Failed to calculate reward amount");
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: metadata.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.beneficiary.to_account_info(),
                },
                &[&[METADATA, &[metadata.bump]]],
            ),
            reward,
        )?;

        // Log result.
        msg!("Height: {}", metadata.height);
        msg!("Reward: {}", reward);
        msg!("Supply: {}", supply);
        emit!(MineEvent {
            signer: ctx.accounts.signer.key(),
            beneficiary: ctx.accounts.beneficiary.key(),
            height: metadata.height,
            reward,
            supply,
            hash,
            nonce,
            difficulty: metadata.difficulty.clone(),
        });

        Ok(())
    }
}

fn validate_hash(
    current_hash: String,
    hash: String,
    signer: Pubkey,
    nonce: u64,
    difficulty: U256,
) -> Result<()> {
    // Validate hash correctness.
    let msg = format!("{}-{}-{}", current_hash, signer, nonce);
    require!(sha256::digest(msg).eq(&hash), ProgramError::HashInvalid);

    // Validate hash difficulty.
    let hash_u256 = U256::parse_str_radix(&hash, RADIX);
    require!(hash_u256.le(&difficulty), ProgramError::HashInvalid);
    Ok(())
}

fn calculate_new_difficulty(t1: i64, t2: i64, difficulty: U256) -> Result<U256> {
    // Calculate time ratio.
    require!(t2.gt(&t1), ProgramError::ClockInvalid);
    let actual_time = t2.saturating_sub(t1) as f64;
    let expected_time = EPOCH_HEIGHT.saturating_mul(HASH_TIME) as f64;
    let time_ratio = actual_time / expected_time;

    // Scale time ratio for integer arithmetic.
    const SCALE_FACTOR: f64 = 1000f64;
    let time_ratio_scaled = U256::from_digit((time_ratio * SCALE_FACTOR) as u64);

    // Calculate new difficulty.
    const SCALE_FACTOR_U256: U256 = U256::from_digit(SCALE_FACTOR as u64);
    let new_difficulty_scaled = difficulty.saturating_mul(time_ratio_scaled);
    let new_difficulty = new_difficulty_scaled.saturating_div(SCALE_FACTOR_U256);

    // Smooth new difficulty to a min/max multiple of the old difficulty.
    let new_difficulty_min = difficulty.saturating_div(SMOOTHING_FACTOR);
    let new_difficulty_max = difficulty.saturating_mul(SMOOTHING_FACTOR);
    let new_difficulty_smoothed = new_difficulty_min.max(new_difficulty_max.min(new_difficulty));
    Ok(new_difficulty_smoothed)
}

fn calculate_supply(height: u128) -> u64 {
    ((SUPPLY_COEFFICIENT as f64 * (height as f64).powf(SUPPLY_EXPONENT))
        * 10f64.powf(TOKEN_DECIMALS as f64)) as u64
}

#[account]
#[derive(Debug, InitSpace)]
pub struct Metadata {
    /// The bump of the metadata account address.
    pub bump: u8,

    /// The current mining difficulty.
    #[max_len(256)]
    pub difficulty: String,

    /// The current hash.
    #[max_len(256)]
    pub hash: String,

    /// The current height of the hash chain.
    pub height: u128,

    /// The mint address of the Ore token.
    pub mint: Pubkey,

    /// The timestamp of the start of the current epoch.
    pub epoch_start_at: i64,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The metadata account.
    #[account(init, seeds = [METADATA], bump, payer = signer, space = 8 + Metadata::INIT_SPACE)]
    pub metadata: Account<'info, Metadata>,

    /// The Ore token mint.
    #[account(init, payer = signer, mint::decimals = TOKEN_DECIMALS, mint::authority = metadata)]
    pub mint: Account<'info, Mint>,

    /// The rent sysvar account.
    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,
}

#[derive(Accounts)]
#[instruction(hash: String, nonce: u64)]
pub struct Mine<'info> {
    /// The signer of the transaction (i.e. the miner).
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The beneficiary token account to mint rewards to.
    #[account(mut, token::mint = mint)]
    pub beneficiary: Account<'info, TokenAccount>,

    /// The metadata account.
    #[account(mut, seeds = [METADATA], bump = metadata.bump, has_one = mint)]
    pub metadata: Account<'info, Metadata>,

    /// The Ore token mint.
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,
}

/// MineEvent logs revelant data about a successful Ore mining transaction.
#[event]
#[derive(Debug)]
pub struct MineEvent {
    /// The signer of the transaction (i.e. the miner).
    pub signer: Pubkey,

    /// The beneficiary token account to which rewards were minted.
    pub beneficiary: Pubkey,

    /// The updated height of the program's hash chain.
    pub height: u128,

    /// The quantity of new Ore tokens that were mined.
    pub reward: u64,

    /// The updated Ore token supply.
    pub supply: u64,

    /// The valid hash provided by the signer.
    pub hash: String,

    /// The nonce provided by the signer.
    pub nonce: u64,

    /// The current mining difficulty.
    pub difficulty: String,
}

#[error_code]
pub enum ProgramError {
    #[msg("The clock time is invalid")]
    ClockInvalid,
    #[msg("The hash is invalid")]
    HashInvalid,
    #[msg("Mining has not started yet")]
    NotStarted,
}

#[cfg(test)]
mod tests {
    use anchor_lang::prelude::Pubkey;
    use bnum::types::U256;
    use rand::prelude::*;

    use crate::{calculate_new_difficulty, calculate_supply, validate_hash};

    #[test]
    fn test_validate_hash_pass() {
        let h1 = sha256::digest("Seed");
        let signer = Pubkey::new_unique();
        let nonce = 10;
        let h2 = sha256::digest(format!("{}-{}-{}", h1, signer, nonce));
        let res = validate_hash(h1, h2, signer, nonce, U256::MAX);
        assert!(res.is_ok());
    }

    #[test]
    fn test_validate_hash_fail() {
        let h1 = sha256::digest("Seed");
        let signer = Pubkey::new_unique();
        let nonce = 10;
        let h2 = String::from("Invalid hash");
        let res = validate_hash(h1, h2, signer, nonce, U256::MAX);
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_hash_fail_difficulty() {
        let h1 = sha256::digest("Seed");
        let signer = Pubkey::new_unique();
        let nonce = 10;
        let h2 = sha256::digest(format!("{}-{}-{}", h1, signer, nonce));
        let res = validate_hash(h1, h2, signer, nonce, U256::MIN);
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_hash_fuzz() {
        let h1 = sha256::digest("Seed");
        let signer = Pubkey::new_unique();
        let mut rng = rand::thread_rng();
        for i in 0..10_000 {
            let nonce = rng.gen::<u64>();
            let h2 = sha256::digest(i.to_string());
            let res = validate_hash(h1.clone(), h2, signer, nonce, U256::MAX);
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_calculate_new_difficulty_stable() {
        let t1 = 0i64;
        let t2 = 86_400i64;
        let difficulty = U256::from_digit(100);
        let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
        assert!(new_difficulty.is_ok());
        let x = new_difficulty.unwrap();
        assert!(x.eq(&U256::from_digit(100)));
    }

    #[test]
    fn test_calculate_new_difficulty_higher() {
        let t1 = 0i64;
        let t2 = 172_800i64;
        let difficulty = U256::from_digit(100);
        let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
        assert!(new_difficulty.is_ok());
        let x = new_difficulty.unwrap();
        assert!(x.eq(&U256::from_digit(200)));
    }

    #[test]
    fn test_calculate_new_difficulty_lower() {
        let t1 = 0i64;
        let t2 = 43_200i64;
        let difficulty = U256::from_digit(100);
        let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
        assert!(new_difficulty.is_ok());
        let x = new_difficulty.unwrap();
        assert!(x.eq(&U256::from_digit(50)));
    }

    #[test]
    fn test_calculate_new_difficulty_max() {
        let t1 = 0i64;
        let t2 = 1_000_000i64;
        let difficulty = U256::from_digit(100);
        let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
        assert!(new_difficulty.is_ok());
        let x = new_difficulty.unwrap();
        assert!(x.eq(&U256::from_digit(400)));
    }

    #[test]
    fn test_calculate_new_difficulty_min() {
        let t1 = 0i64;
        let t2 = 1i64;
        let difficulty = U256::from_digit(100);
        let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
        assert!(new_difficulty.is_ok());
        let x = new_difficulty.unwrap();
        assert!(x.eq(&U256::from_digit(25)));
    }

    #[test]
    fn test_calculate_new_difficulty_err() {
        let t1 = 10i64;
        let t2 = 5i64;
        let difficulty = U256::from_digit(100);
        let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
        assert!(new_difficulty.is_err());
    }

    #[test]
    fn test_calculate_supply() {
        let s1 = calculate_supply(1);
        let s10 = calculate_supply(10);
        let s100 = calculate_supply(100);
        assert!(s1.eq(&4_848_400_000_000));
        assert!(s10.eq(&20_118_631_803_084));
        assert!(s100.eq(&83_483_075_989_621));
    }
}

fn estimate_size(x: u32) -> u32 {
    assert!(x < 4096);

    if x < 256 {
        if x < 128 {
            return 1;
        } else {
            return 3;
        }
    } else if x < 1024 {
        if x > 1022 {
            return 4;
        } else {
            return 5;
        }
    } else {
        if x < 2048 {
            return 7;
        } else {
            return 9;
        }
    }
}

#[cfg(kani)]
mod verification {
    use super::*;

    #[kani::proof]
    pub fn check() {
        // let x: u32 = kani::any();
        // kani::assume(x < 4096);
        // let y = estimate_size(x);
        // assert!(y < 10);
    }
}
