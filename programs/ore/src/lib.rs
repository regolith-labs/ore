use std::mem::size_of;

use anchor_lang::{
    prelude::*,
    solana_program::{
        hash::{hashv, Hash},
        system_program, sysvar,
    },
};
use anchor_spl::token::{self, Mint, MintTo, TokenAccount};

declare_id!("CeJShZEAzBLwtcLQvbZc7UT38e4nUTn63Za5UFyYYDTS");

// TODO Set this to a reasonable value.
pub const DIFFICULTY: Hash = Hash::new_from_array([
    0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
]);

// TODO Set this before deployment
/// The time after which mining can happen.
pub const START_AT: i64 = 0;

pub const EPOCH_DURATION: i64 = 60;

pub const EXPECTED_EPOCH_REWARDS: u64 = 10u64.pow(TOKEN_DECIMALS as u32); // 1 ORE / epoch

pub const SMOOTHING_FACTOR: u64 = 256;

/// The decimal precision of the Ore token.
// If we use a decimal precision of 16, we can fit 10_000_000_000_000_000 (10 quadrillion) hashes in each minute.
// This is sufficiently far beyond what Solana is capable of processing.
// Max supply would still be very large and take millions of years to reach at a rate of 1 ORE / minute.
// We will not have to implement a variable difficulty to maintain the 1 ORE / min average.
// If token decimals were only 8, we likely would need a variable difficulty at some point.
pub const TOKEN_DECIMALS: u8 = 9;

pub const BUS_COUNT: u64 = 8;

// TODO Use 8,9,or 10 decimals. Test and run math to see which one makes the most sense.
// TODO Track number of successful hashes per bus and add limit to # hashes/bus exceeding EXPECTED_EPOCH_REWARDS/NUMBER_OF_BUSSES.
//      Rewards per epoch can exceed target limit. But hashes cannot exceed theoretical maximum.
//      Eg. scenario where reward_rate = 1, cannot go lower, and the network is submitting enough hashes to push issuance rate above 1 ORE / epoch.
//      In this case, each valid hash is earning the smallest reward possible.
//      By hard limitting the number of hashes per bus per epoch, we prevent the 1 ORE/epoch limit from being fundamentally broken.

#[program]
mod ore {
    use super::*;

    /// Initializes the program. Can only be executed once.
    pub fn initialize_metadata(ctx: Context<InitializeMetadata>) -> Result<()> {
        ctx.accounts.metadata.bump = ctx.bumps.metadata;
        ctx.accounts.metadata.reward_rate = 10u64.pow(TOKEN_DECIMALS.saturating_div(2) as u32);
        ctx.accounts.metadata.mint = ctx.accounts.mint.key();
        Ok(())
    }

    /// Initializes the program. Can only be executed once.
    pub fn initialize_busses(ctx: Context<InitializeBusses>) -> Result<()> {
        ctx.accounts.bus_0.bump = ctx.bumps.bus_0;
        ctx.accounts.bus_0.id = 0;
        ctx.accounts.bus_1.bump = ctx.bumps.bus_1;
        ctx.accounts.bus_1.id = 1;
        ctx.accounts.bus_2.bump = ctx.bumps.bus_2;
        ctx.accounts.bus_2.id = 2;
        ctx.accounts.bus_3.bump = ctx.bumps.bus_3;
        ctx.accounts.bus_3.id = 3;
        ctx.accounts.bus_4.bump = ctx.bumps.bus_4;
        ctx.accounts.bus_4.id = 4;
        ctx.accounts.bus_5.bump = ctx.bumps.bus_5;
        ctx.accounts.bus_5.id = 5;
        ctx.accounts.bus_6.bump = ctx.bumps.bus_6;
        ctx.accounts.bus_6.id = 6;
        ctx.accounts.bus_7.bump = ctx.bumps.bus_7;
        ctx.accounts.bus_7.id = 7;
        Ok(())
    }

    pub fn register_miner(ctx: Context<RegisterMiner>) -> Result<()> {
        let miner = &mut ctx.accounts.miner;
        miner.authority = ctx.accounts.signer.key();
        miner.bump = ctx.bumps.miner;
        miner.hash = hashv(&[&ctx.accounts.signer.key().to_bytes()]);
        Ok(())
    }

    pub fn start_epoch(ctx: Context<StartEpoch>) -> Result<()> {
        // Validate epoch has ended.
        let clock = Clock::get().unwrap();
        let metadata = &mut ctx.accounts.metadata;
        let epoch_end_at = metadata.epoch_start_at.saturating_add(EPOCH_DURATION);
        require!(
            clock.unix_timestamp.ge(&epoch_end_at),
            ProgramError::ClockInvalid
        );

        // Calculate total rewards issued during the epoch.
        let bus_0 = &mut ctx.accounts.bus_0;
        let bus_1 = &mut ctx.accounts.bus_1;
        let bus_2 = &mut ctx.accounts.bus_2;
        let bus_3 = &mut ctx.accounts.bus_3;
        let bus_4 = &mut ctx.accounts.bus_4;
        let bus_5 = &mut ctx.accounts.bus_5;
        let bus_6 = &mut ctx.accounts.bus_6;
        let bus_7 = &mut ctx.accounts.bus_7;
        let total_epoch_rewards = bus_0
            .rewards
            .saturating_add(bus_1.rewards)
            .saturating_add(bus_2.rewards)
            .saturating_add(bus_3.rewards)
            .saturating_add(bus_4.rewards)
            .saturating_add(bus_5.rewards)
            .saturating_add(bus_6.rewards)
            .saturating_add(bus_7.rewards);

        // Update the reward amount for the next epoch.
        metadata.reward_rate = calculate_new_reward_rate(metadata.reward_rate, total_epoch_rewards);

        // Reset state for new epoch.
        bus_0.hashes = 0;
        bus_1.hashes = 0;
        bus_2.hashes = 0;
        bus_3.hashes = 0;
        bus_4.hashes = 0;
        bus_5.hashes = 0;
        bus_6.hashes = 0;
        bus_7.hashes = 0;
        bus_0.hashes = 0;
        bus_1.rewards = 0;
        bus_2.rewards = 0;
        bus_3.rewards = 0;
        bus_4.rewards = 0;
        bus_5.rewards = 0;
        bus_6.rewards = 0;
        bus_7.rewards = 0;
        metadata.epoch_start_at = clock.unix_timestamp;
        Ok(())
    }

    pub fn mine(ctx: Context<Mine>, hash: Hash, nonce: u64) -> Result<()> {
        // Validate epoch is active.
        let clock = Clock::get().unwrap();
        let metadata = &mut ctx.accounts.metadata;
        let epoch_end_at = metadata.epoch_start_at.saturating_add(EPOCH_DURATION);
        require!(
            clock.unix_timestamp.lt(&epoch_end_at),
            ProgramError::EpochNotActive
        );

        // Validate hash.
        let miner = &mut ctx.accounts.miner;
        validate_hash(
            miner.hash.clone(),
            hash.clone(),
            ctx.accounts.signer.key(),
            nonce,
            DIFFICULTY,
        )?;

        // Update state.
        ctx.accounts.bus.hashes = ctx.accounts.bus.hashes.saturating_add(1);
        ctx.accounts.bus.rewards = ctx
            .accounts
            .bus
            .rewards
            .saturating_add(metadata.reward_rate);
        miner.hash = hash.clone();

        // Error if this bus has already processed its quota of hashes for the epoch.
        require!(
            ctx.accounts
                .bus
                .hashes
                .le(&EXPECTED_EPOCH_REWARDS.saturating_div(BUS_COUNT)),
            // TODO Needs a dedicated error
            ProgramError::HashInvalid
        );

        // Mint reward to beneficiary.
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
            metadata.reward_rate,
        )?;

        Ok(())
    }
}

fn validate_hash(
    current_hash: Hash,
    hash: Hash,
    signer: Pubkey,
    nonce: u64,
    difficulty: Hash,
) -> Result<()> {
    // Validate hash correctness.
    let bytes = [
        current_hash.to_bytes().as_slice(),
        signer.to_bytes().as_slice(),
        nonce.to_be_bytes().as_slice(),
    ]
    .concat();
    let hash_ = hashv(&[&bytes]);
    require!(hash.eq(&hash_), ProgramError::HashInvalid);

    // Validate hash difficulty.
    require!(hash.le(&difficulty), ProgramError::HashInvalid);

    Ok(())
}

fn calculate_new_reward_rate(current_rate: u64, epoch_rewards: u64) -> u64 {
    // Avoid division by zero. Leave the reward rate unchanged.
    if epoch_rewards.eq(&0) {
        return current_rate;
    }

    // Calculate new rate.
    msg!("Current rate: {}", current_rate);
    msg!("Epoch rewards: {}", epoch_rewards);
    msg!("Expected rewards: {}", EXPECTED_EPOCH_REWARDS);
    let new_rate = (current_rate as u128)
        .saturating_mul(EXPECTED_EPOCH_REWARDS as u128)
        .saturating_div(epoch_rewards as u128) as u64;
    msg!("New rate: {}", new_rate);

    // Smooth reward rate to not change by more than a constant factor from one epoch to the next.
    let new_rate_min = current_rate.saturating_div(SMOOTHING_FACTOR);
    let new_rate_max = current_rate.saturating_mul(SMOOTHING_FACTOR);
    let new_rate_smoothed = new_rate_min.max(new_rate_max.min(new_rate));
    msg!("New rate min: {}", new_rate_min);
    msg!("New rate max: {}", new_rate_max);
    msg!("New rate smoothed: {}", new_rate_smoothed);

    // Prevent new reward from reaching 0 and return.
    new_rate_smoothed.max(1)
}

/// The seed of the Bus account PDA.
pub const BUS: &[u8] = b"bus";

/// The seed of the Metadata account PDA.
pub const METADATA: &[u8] = b"metadata";

/// The seed of the Miner account PDA.
pub const MINER: &[u8] = b"miner";

#[account]
#[derive(Debug)]
pub struct Metadata {
    /// The bump of the metadata PDA.
    pub bump: u8,

    /// The mint address of the Ore token.
    pub mint: Pubkey,

    /// The timestamp of the start of the current epoch.
    pub epoch_start_at: i64,

    /// Reweard
    pub reward_rate: u64,
}

#[account]
#[derive(Debug)]
pub struct Miner {
    /// The bump of the miner PDA.
    pub bump: u8,

    /// The account authorized to hash this chain.
    pub authority: Pubkey,

    /// The miner's current hash.
    pub hash: Hash,
}

/// Bus is an account used to track rewards issued during an epoch.
/// There are 8 bus accounts to provide parallelism and reduce write lock contention.
#[account]
#[derive(Debug)]
pub struct Bus {
    /// The bump of the counter PDA.
    pub bump: u8,

    /// The ID of this counter account.
    pub id: u8,

    /// The count of rewards issued this epoch.
    pub rewards: u64,

    /// The count of valid hashes that have been submitted on this bus this epoch.
    pub hashes: u64,
}

#[derive(Accounts)]
pub struct InitializeMetadata<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The metadata account.
    #[account(init, seeds = [METADATA], bump, payer = signer, space = 8 + size_of::<Metadata>())]
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
pub struct InitializeBusses<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Bus account 0.
    #[account(init, seeds = [BUS, &[0]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_0: Account<'info, Bus>,

    /// Bus account 1.
    #[account(init, seeds = [BUS, &[1]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_1: Account<'info, Bus>,

    /// Bus account 2.
    #[account(init, seeds = [BUS, &[2]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_2: Account<'info, Bus>,

    /// Bus account 3.
    #[account(init, seeds = [BUS, &[3]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_3: Account<'info, Bus>,

    /// Bus account 4.
    #[account(init, seeds = [BUS, &[4]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_4: Account<'info, Bus>,

    /// Bus account 5.
    #[account(init, seeds = [BUS, &[5]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_5: Account<'info, Bus>,

    /// Bus account 6.
    #[account(init, seeds = [BUS, &[6]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_6: Account<'info, Bus>,

    /// Bus account 7.
    #[account(init, seeds = [BUS, &[7]], bump, payer = signer, space = 8 + size_of::<Bus>())]
    pub bus_7: Account<'info, Bus>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

/// RegisterMiner registers a new miner with the Ore program and starts a new hash chain for them to mine.
#[derive(Accounts)]
pub struct RegisterMiner<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The miner account.
    #[account(init, seeds = [MINER, signer.key().as_ref()], bump, payer = signer, space = 8 + size_of::<Miner>())]
    pub miner: Account<'info, Miner>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct StartEpoch<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Counter account 0.
    #[account(mut, seeds = [BUS, &[0]], bump)]
    pub bus_0: Account<'info, Bus>,

    /// Bus account 1.
    #[account(mut, seeds = [BUS, &[1]], bump)]
    pub bus_1: Account<'info, Bus>,

    /// Bus account 2.
    #[account(mut, seeds = [BUS, &[2]], bump)]
    pub bus_2: Account<'info, Bus>,

    /// Bus account 3.
    #[account(mut, seeds = [BUS, &[3]], bump)]
    pub bus_3: Account<'info, Bus>,

    /// Bus account 4.
    #[account(mut, seeds = [BUS, &[4]], bump)]
    pub bus_4: Account<'info, Bus>,

    /// Bus account 5.
    #[account(mut, seeds = [BUS, &[5]], bump)]
    pub bus_5: Account<'info, Bus>,

    /// Bus account 6.
    #[account(mut, seeds = [BUS, &[6]], bump)]
    pub bus_6: Account<'info, Bus>,

    /// Bus account 7.
    #[account(mut, seeds = [BUS, &[7]], bump)]
    pub bus_7: Account<'info, Bus>,

    /// The metadata account.
    #[account(mut, seeds = [METADATA], bump)]
    pub metadata: Account<'info, Metadata>,
}

// TODO Bytes, not strings
#[derive(Accounts)]
#[instruction(hash: Hash, nonce: u64)]
pub struct Mine<'info> {
    /// The signer of the transaction (i.e. the miner).
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The beneficiary token account to mint rewards to.
    #[account(mut, token::mint = mint)]
    pub beneficiary: Account<'info, TokenAccount>,

    /// A bus account for tracking epoch rewards.
    #[account(mut)]
    pub bus: Account<'info, Bus>,

    /// The metadata account.
    #[account(seeds = [METADATA], bump = metadata.bump, has_one = mint)]
    pub metadata: Account<'info, Metadata>,

    /// The metadata account.
    #[account(mut, seeds = [MINER, signer.key().as_ref()], bump = miner.bump, constraint = signer.key().eq(&miner.authority))]
    pub miner: Account<'info, Miner>,

    /// The Ore token mint account.
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

    /// The quantity of new Ore tokens that were mined.
    pub reward: u64,

    /// The updated Ore token supply.
    pub supply: u64,

    /// The valid hash provided by the signer.
    pub hash: Hash,

    /// The current mining difficulty.
    pub difficulty: Hash,

    /// The nonce provided by the signer.
    pub nonce: u64,
}

#[error_code]
pub enum ProgramError {
    #[msg("The clock time is invalid")]
    ClockInvalid,
    #[msg("The hash is invalid")]
    HashInvalid,
    #[msg("Mining has not started yet")]
    NotStarted,
    #[msg("The epoch has ended and needs to be reset")]
    EpochNotActive,
}

#[cfg(test)]
mod tests {
    // use anchor_lang::prelude::Pubkey;
    // use bnum::types::U256;
    // use rand::prelude::*;

    // use crate::validate_hash;

    #[test]
    fn test_validate_hash_pass() {
        // let h1 = sha256::digest("Seed");
        // let signer = Pubkey::new_unique();
        // let nonce = 10;
        // let h2 = sha256::digest(format!("{}-{}-{}", h1, signer, nonce));
        // let res = validate_hash(h1, h2, signer, nonce, U256::MAX);
        // assert!(res.is_ok());
    }

    #[test]
    fn test_validate_hash_fail() {
        // let h1 = sha256::digest("Seed");
        // let signer = Pubkey::new_unique();
        // let nonce = 10;
        // let h2 = String::from("Invalid hash");
        // let res = validate_hash(h1, h2, signer, nonce, U256::MAX);
        // assert!(res.is_err());
    }

    #[test]
    fn test_validate_hash_fail_difficulty() {
        // let h1 = sha256::digest("Seed");
        // let signer = Pubkey::new_unique();
        // let nonce = 10;
        // let h2 = sha256::digest(format!("{}-{}-{}", h1, signer, nonce));
        // let res = validate_hash(h1, h2, signer, nonce, U256::MIN);
        // assert!(res.is_err());
    }

    #[test]
    fn test_validate_hash_fuzz() {
        // let h1 = sha256::digest("Seed");
        // let signer = Pubkey::new_unique();
        // let mut rng = rand::thread_rng();
        // for i in 0..10_000 {
        //     let nonce = rng.gen::<u64>();
        //     let h2 = sha256::digest(i.to_string());
        //     let res = validate_hash(h1.clone(), h2, signer, nonce, U256::MAX);
        //     assert!(res.is_err());
        // }
    }

    // #[test]
    // fn test_calculate_new_difficulty_stable() {
    //     let t1 = 0i64;
    //     let t2 = 86_400i64;
    //     let difficulty = U256::from_digit(100);
    //     let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
    //     assert!(new_difficulty.is_ok());
    //     let x = new_difficulty.unwrap();
    //     assert!(x.eq(&U256::from_digit(100)));
    // }

    // #[test]
    // fn test_calculate_new_difficulty_higher() {
    //     let t1 = 0i64;
    //     let t2 = 172_800i64;
    //     let difficulty = U256::from_digit(100);
    //     let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
    //     assert!(new_difficulty.is_ok());
    //     let x = new_difficulty.unwrap();
    //     assert!(x.eq(&U256::from_digit(200)));
    // }

    // #[test]
    // fn test_calculate_new_difficulty_lower() {
    //     let t1 = 0i64;
    //     let t2 = 43_200i64;
    //     let difficulty = U256::from_digit(100);
    //     let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
    //     assert!(new_difficulty.is_ok());
    //     let x = new_difficulty.unwrap();
    //     assert!(x.eq(&U256::from_digit(50)));
    // }

    // #[test]
    // fn test_calculate_new_difficulty_max() {
    //     let t1 = 0i64;
    //     let t2 = 1_000_000i64;
    //     let difficulty = U256::from_digit(100);
    //     let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
    //     assert!(new_difficulty.is_ok());
    //     let x = new_difficulty.unwrap();
    //     assert!(x.eq(&U256::from_digit(400)));
    // }

    // #[test]
    // fn test_calculate_new_difficulty_min() {
    //     let t1 = 0i64;
    //     let t2 = 1i64;
    //     let difficulty = U256::from_digit(100);
    //     let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
    //     assert!(new_difficulty.is_ok());
    //     let x = new_difficulty.unwrap();
    //     assert!(x.eq(&U256::from_digit(25)));
    // }

    // #[test]
    // fn test_calculate_new_difficulty_err() {
    //     let t1 = 10i64;
    //     let t2 = 5i64;
    //     let difficulty = U256::from_digit(100);
    //     let new_difficulty = calculate_new_difficulty(t1, t2, difficulty);
    //     assert!(new_difficulty.is_err());
    // }

    // #[test]
    // fn test_calculate_supply() {
    //     let s1 = calculate_supply(1);
    //     let s10 = calculate_supply(10);
    //     let s100 = calculate_supply(100);
    //     assert!(s1.eq(&4_848_400_000_000));
    //     assert!(s10.eq(&20_118_631_803_084));
    //     assert!(s100.eq(&83_483_075_989_621));
    // }
}
