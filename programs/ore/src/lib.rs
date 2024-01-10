use std::mem::size_of;

use anchor_lang::{
    prelude::*,
    solana_program::{
        hash::{hashv, Hash},
        system_program, sysvar,
    },
};
use anchor_spl::{
    associated_token,
    token::{self, Mint, MintTo, TokenAccount},
};

declare_id!("CeJShZEAzBLwtcLQvbZc7UT38e4nUTn63Za5UFyYYDTS");

// TODO Set this to a reasonable value.
pub const DIFFICULTY: Hash = Hash::new_from_array([
    0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
]);

// TODO Set this before deployment
/// The start time for which mining can begin.
pub const START_AT: i64 = 0;

/// The duration of an epoch, in units of seconds.
pub const EPOCH_DURATION: i64 = 60;

/// One ORE token, denominated in indivisible units of nanoORE.
pub const ONE_ORE: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// The quantity of ORE expected to be minted per epoch, in units of indivisible nanoORE.
/// Inflation rate = 1 ORE / epoch
pub const EXPECTED_EPOCH_REWARDS: u64 = ONE_ORE; // 10u64.pow(TOKEN_DECIMALS as u32);

/// The smoothing factor for reward rate changes. The reward rate cannot change by more or less
/// than factor of this constant from one epoch to the next.
pub const SMOOTHING_FACTOR: u64 = 8;

/// The decimal precision of the ORE token.
/// Using SI prefixes, the smallest indivisible unit of ORE is a nanoORE.
/// 1 nanoORE = 0.000000001 ORE = one billionth of an ORE
pub const TOKEN_DECIMALS: u8 = 9;

/// The number of available bus accounts, for parallelizing mine operations.
pub const BUS_COUNT: u8 = 8;

#[program]
mod ore {
    use super::*;

    /// Initializes the metadata account. Can only be invoked once.
    pub fn initialize_metadata(ctx: Context<InitializeMetadata>) -> Result<()> {
        ctx.accounts.metadata.bump = ctx.bumps.metadata;
        ctx.accounts.metadata.reward_rate = 10u64.pow(TOKEN_DECIMALS.saturating_div(2) as u32);
        ctx.accounts.metadata.mint = ctx.accounts.mint.key();
        Ok(())
    }

    /// Initializes the bus accounts. Can only be invoked once.
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

    /// Initializes an associated token account for a bus. Can only be invoked once per bus.
    pub fn initialize_bus_tokens(_ctx: Context<InitializeBusTokens>) -> Result<()> {
        Ok(())
    }

    /// Initializes a proof account for a new miner.
    pub fn register(ctx: Context<Register>) -> Result<()> {
        let proof = &mut ctx.accounts.proof;
        proof.authority = ctx.accounts.signer.key();
        proof.bump = ctx.bumps.proof;
        proof.hash = hashv(&[&ctx.accounts.signer.key().to_bytes()]);
        Ok(())
    }

    /// Updates the reward rate and starts the new epoch.
    pub fn reset_epoch(ctx: Context<ResetEpoch>) -> Result<()> {
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
        metadata.epoch_start_at = clock.unix_timestamp;

        // Reset bus accounts.
        let mint = &ctx.accounts.mint;
        let metadata = &ctx.accounts.metadata;
        let token_program = &ctx.accounts.token_program;
        reset_bus(
            bus_0,
            &mut ctx.accounts.bus_0_tokens,
            mint,
            metadata,
            token_program,
        )?;
        reset_bus(
            bus_1,
            &mut ctx.accounts.bus_1_tokens,
            mint,
            metadata,
            token_program,
        )?;
        reset_bus(
            bus_2,
            &mut ctx.accounts.bus_2_tokens,
            mint,
            metadata,
            token_program,
        )?;
        reset_bus(
            bus_3,
            &mut ctx.accounts.bus_3_tokens,
            mint,
            metadata,
            token_program,
        )?;
        reset_bus(
            bus_4,
            &mut ctx.accounts.bus_4_tokens,
            mint,
            metadata,
            token_program,
        )?;
        reset_bus(
            bus_5,
            &mut ctx.accounts.bus_5_tokens,
            mint,
            metadata,
            token_program,
        )?;
        reset_bus(
            bus_6,
            &mut ctx.accounts.bus_6_tokens,
            mint,
            metadata,
            token_program,
        )?;
        reset_bus(
            bus_7,
            &mut ctx.accounts.bus_7_tokens,
            mint,
            metadata,
            token_program,
        )?;

        Ok(())
    }

    /// Mints new Ore tokens to miners who submit valid hashes.
    pub fn mine(ctx: Context<Mine>, hash: Hash, nonce: u64) -> Result<()> {
        // Validate epoch is active.
        let clock = Clock::get().unwrap();
        let metadata = &mut ctx.accounts.metadata;
        let epoch_end_at = metadata.epoch_start_at.saturating_add(EPOCH_DURATION);
        require!(
            clock.unix_timestamp.lt(&epoch_end_at),
            ProgramError::EpochNeedsReset
        );

        // Validate hash.
        let proof = &mut ctx.accounts.proof;
        validate_hash(
            proof.hash.clone(),
            hash.clone(),
            ctx.accounts.signer.key(),
            nonce,
            DIFFICULTY,
        )?;

        // Update state.
        let bus = &mut ctx.accounts.bus;
        bus.hashes = bus.hashes.saturating_add(1);
        bus.rewards = bus.rewards.saturating_add(metadata.reward_rate);
        proof.hash = hash.clone();

        // Error if this bus has reached its hash quota for the epoch.
        //
        // This constraint places an upper bound on the total number of hashes that can be
        // processed during any given epoch. This limit is necessary to maintain a constant
        // 1 ORE/epoch inflation rate in the potential future scenario where global hashpower
        // is so great that the reward rate is pushed to its minimum non-divisible value
        // of 1 (one nanoORE) and miners are submitting >10^9 mine ops per epoch.
        //
        // In this extreme scenario, we err to the side of hardcapping inflation at 1 ORE/epoch.
        // Even if all busses are saturated, miners can still avoid starvation by waiting until
        // the next epoch and submitting transactions with a market rate priority fee.
        //
        // Even if Solana achieves 1M real TPS and all transactions are mine ops, the network
        // would still only be able to process (60 * 1,000,000) = 60,000,000 hashes per epoch.
        // That is, Solana should reach its network saturation point long before this quota
        // is enforced here.
        require!(
            bus.hashes
                .le(&EXPECTED_EPOCH_REWARDS.saturating_div(BUS_COUNT as u64)),
            ProgramError::BusQuotaFilled
        );

        // Issue tokens from bus to beneficiary.
        let bus_tokens = &ctx.accounts.bus_tokens;
        require!(
            bus_tokens.amount.ge(&metadata.reward_rate),
            ProgramError::BusInsufficientFunds
        );
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: bus_tokens.to_account_info(),
                    to: ctx.accounts.beneficiary.to_account_info(),
                    authority: bus.to_account_info(),
                },
                &[&[BUS, &[bus.id], &[bus.bump]]],
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

    // Calculate new reward rate.
    let new_rate = (current_rate as u128)
        .saturating_mul(EXPECTED_EPOCH_REWARDS as u128)
        .saturating_div(epoch_rewards as u128) as u64;

    // Smooth reward rate to not change by more than a constant factor from one epoch to the next.
    let new_rate_min = current_rate.saturating_div(SMOOTHING_FACTOR);
    let new_rate_max = current_rate.saturating_mul(SMOOTHING_FACTOR);
    let new_rate_smoothed = new_rate_min.max(new_rate_max.min(new_rate));

    // Prevent reward rate from reaching 0 and return.
    new_rate_smoothed.max(1)
}

fn reset_bus<'info>(
    bus: &mut Account<Bus>,
    bus_tokens: &mut Account<'info, TokenAccount>,
    mint: &Account<'info, Mint>,
    metadata: &Account<'info, Metadata>,
    token_program: &Program<'info, token::Token>,
) -> Result<()> {
    // Reset bus state.
    bus.hashes = 0;
    bus.rewards = 0;

    // Top up bus account with 1 ORE.
    let amount = ONE_ORE.saturating_sub(bus_tokens.amount);
    if amount.gt(&0) {
        token::mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    authority: metadata.to_account_info(),
                    mint: mint.to_account_info(),
                    to: bus_tokens.to_account_info(),
                },
                &[&[METADATA, &[metadata.bump]]],
            ),
            amount,
        )?;
    }

    Ok(())
}

/// The seed of the bus account PDA.
pub const BUS: &[u8] = b"bus";

/// The seed of the metadata account PDA.
pub const METADATA: &[u8] = b"metadata";

/// The seed of the proof account PDA.
pub const PROOF: &[u8] = b"proof";

/// Bus is an account type used to track the number of processed hashes and issued rewards
/// during an epoch. There are 8 bus accounts to provide sufficient parallelism for mine ops
/// and reduce write lock contention.
#[account]
#[derive(Debug)]
pub struct Bus {
    /// The bump of the bus account PDA.
    pub bump: u8,

    /// The ID of the bus account.
    pub id: u8,

    /// The count of valid hashes submited to this bus in the current epoch.
    pub hashes: u64,

    /// The count of rewards issued by this bus in the current epoch.
    pub rewards: u64,
}

/// Metadata is an account type used to track global program variables.
#[account]
#[derive(Debug)]
pub struct Metadata {
    /// The bump of the metadata account PDA.
    pub bump: u8,

    /// The mint address of the ORE token.
    pub mint: Pubkey,

    /// The timestamp of the start of the current epoch.
    pub epoch_start_at: i64,

    /// The reward rate to payout to miners for submiting valid hashes.
    pub reward_rate: u64,
}

// TODO Track lifetime rewards?
// TODO Track lifetime hashes?
/// Proof is an account type used to track a miner's hash chain.
#[account]
#[derive(Debug)]
pub struct Proof {
    /// The bump of the proof account PDA.
    pub bump: u8,

    /// The account (i.e. miner) authorized to use this proof.
    pub authority: Pubkey,

    /// The proof's current hash.
    pub hash: Hash,
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

    /// The metadata account.
    #[account(has_one = mint)]
    pub metadata: Account<'info, Metadata>,

    /// The Ore token mint account.
    #[account()]
    pub mint: Account<'info, Mint>,

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

#[derive(Accounts)]
pub struct InitializeBusTokens<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The metadata account.
    #[account(has_one = mint)]
    pub metadata: Account<'info, Metadata>,

    /// The Ore token mint account.
    #[account()]
    pub mint: Account<'info, Mint>,

    /// The bus account.
    #[account()]
    pub bus: Account<'info, Bus>,

    /// The bus token account.
    #[account(init, associated_token::mint = mint, associated_token::authority = bus, payer = signer)]
    pub bus_tokens: Account<'info, TokenAccount>,

    /// The rent sysvar account.
    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,

    /// The SPL associated token program.
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

/// Register registers a new miner and initialize a proof account for them.
#[derive(Accounts)]
pub struct Register<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The proof account.
    #[account(init, seeds = [PROOF, signer.key().as_ref()], bump, payer = signer, space = 8 + size_of::<Proof>())]
    pub proof: Account<'info, Proof>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

// ResetEpoch adjusts the reward rate based on global hashpower and begins the new epoch.
#[derive(Accounts)]
pub struct ResetEpoch<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Bus account 0.
    #[account(mut, seeds = [BUS, &[0]], bump = bus_0.bump)]
    pub bus_0: Box<Account<'info, Bus>>,

    /// Bus account 1.
    #[account(mut, seeds = [BUS, &[1]], bump = bus_1.bump)]
    pub bus_1: Box<Account<'info, Bus>>,

    /// Bus account 2.
    #[account(mut, seeds = [BUS, &[2]], bump = bus_2.bump)]
    pub bus_2: Box<Account<'info, Bus>>,

    /// Bus account 3.
    #[account(mut, seeds = [BUS, &[3]], bump = bus_3.bump)]
    pub bus_3: Box<Account<'info, Bus>>,

    /// Bus account 4.
    #[account(mut, seeds = [BUS, &[4]], bump = bus_4.bump)]
    pub bus_4: Box<Account<'info, Bus>>,

    /// Bus account 5.
    #[account(mut, seeds = [BUS, &[5]], bump = bus_5.bump)]
    pub bus_5: Box<Account<'info, Bus>>,

    /// Bus account 6.
    #[account(mut, seeds = [BUS, &[6]], bump = bus_6.bump)]
    pub bus_6: Box<Account<'info, Bus>>,

    /// Bus account 7.
    #[account(mut, seeds = [BUS, &[7]], bump = bus_7.bump)]
    pub bus_7: Box<Account<'info, Bus>>,

    /// Bus token account 0.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_0)]
    pub bus_0_tokens: Box<Account<'info, TokenAccount>>,

    /// Bus token account 1.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_1)]
    pub bus_1_tokens: Box<Account<'info, TokenAccount>>,

    /// Bus token account 2.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_2)]
    pub bus_2_tokens: Box<Account<'info, TokenAccount>>,

    /// Bus token account 3.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_3)]
    pub bus_3_tokens: Box<Account<'info, TokenAccount>>,

    /// Bus token account 4.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_4)]
    pub bus_4_tokens: Box<Account<'info, TokenAccount>>,

    /// Bus token account 5.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_5)]
    pub bus_5_tokens: Box<Account<'info, TokenAccount>>,

    /// Bus token account 6.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_6)]
    pub bus_6_tokens: Box<Account<'info, TokenAccount>>,

    /// Bus token account 7.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus_7)]
    pub bus_7_tokens: Box<Account<'info, TokenAccount>>,

    /// The Ore token mint account.
    #[account(mut)]
    pub mint: Account<'info, Mint>,

    /// The metadata account.
    #[account(mut, seeds = [METADATA], bump = metadata.bump, has_one = mint)]
    pub metadata: Account<'info, Metadata>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,

    /// The SPL associated token program.
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

#[derive(Accounts)]
#[instruction(hash: Hash, nonce: u64)]
pub struct Mine<'info> {
    /// The signer of the transaction (i.e. the miner).
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The beneficiary token account to mint rewards to.
    #[account(mut, token::mint = mint)]
    pub beneficiary: Account<'info, TokenAccount>,

    /// A bus account.
    #[account(mut)]
    pub bus: Account<'info, Bus>,

    /// The bus' token account.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus)]
    pub bus_tokens: Account<'info, TokenAccount>,

    /// The metadata account.
    #[account(seeds = [METADATA], bump = metadata.bump, has_one = mint)]
    pub metadata: Account<'info, Metadata>,

    /// The proof account.
    #[account(mut, seeds = [PROOF, signer.key().as_ref()], bump = proof.bump, constraint = signer.key().eq(&proof.authority))]
    pub proof: Account<'info, Proof>,

    /// The Ore token mint account.
    #[account()]
    pub mint: Account<'info, Mint>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,
}

/// MineEvent logs revelant data about a successful Ore mining transaction.
#[event]
#[derive(Debug)]
pub struct MineEvent {
    /// The signer of the transaction (i.e. the proof).
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
    EpochNeedsReset,
    #[msg("This bus hash reached its hash quota for this epoch")]
    BusQuotaFilled,
    #[msg("This bus does not have enough tokens to pay the reward")]
    BusInsufficientFunds,
}

#[cfg(test)]
mod tests {
    use anchor_lang::{
        prelude::Pubkey,
        solana_program::hash::{hashv, Hash},
    };

    use crate::{
        calculate_new_reward_rate, validate_hash, EXPECTED_EPOCH_REWARDS, SMOOTHING_FACTOR,
    };

    #[test]
    fn test_validate_hash_pass() {
        let h1 = Hash::new_from_array([1; 32]);
        let signer = Pubkey::new_unique();
        let nonce = 10u64;
        let difficulty = Hash::new_from_array([255; 32]);
        let h2 = hashv(&[
            h1.to_bytes().as_slice(),
            signer.to_bytes().as_slice(),
            nonce.to_be_bytes().as_slice(),
        ]);
        let res = validate_hash(h1, h2, signer, nonce, difficulty);
        assert!(res.is_ok());
    }

    #[test]
    fn test_validate_hash_fail() {
        let h1 = Hash::new_from_array([1; 32]);
        let signer = Pubkey::new_unique();
        let nonce = 10u64;
        let difficulty = Hash::new_from_array([255; 32]);
        let h2 = Hash::new_from_array([2; 32]);
        let res = validate_hash(h1, h2, signer, nonce, difficulty);
        assert!(res.is_err());
    }

    #[test]
    fn test_validate_hash_fail_difficulty() {
        let h1 = Hash::new_from_array([1; 32]);
        let signer = Pubkey::new_unique();
        let nonce = 10u64;
        let difficulty = Hash::new_from_array([0; 32]);
        let h2 = hashv(&[
            h1.to_bytes().as_slice(),
            signer.to_bytes().as_slice(),
            nonce.to_be_bytes().as_slice(),
        ]);
        let res = validate_hash(h1, h2, signer, nonce, difficulty);
        assert!(res.is_err());
    }

    #[test]
    fn test_calculate_new_reward_rate_stable() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, EXPECTED_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_no_chage() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 0);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            EXPECTED_EPOCH_REWARDS.saturating_add(1_000_000),
        );
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_higher() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            EXPECTED_EPOCH_REWARDS.saturating_sub(1_000_000),
        );
        assert!(new_rate.gt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_max_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 1);
        assert!(new_rate.eq(&current_rate.saturating_mul(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, u64::MAX);
        assert!(new_rate.eq(&current_rate.saturating_div(SMOOTHING_FACTOR)));
    }
}
