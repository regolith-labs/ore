use std::mem::size_of;

use anchor_lang::{
    prelude::*,
    solana_program::{
        hash::{hashv, Hash},
        slot_hashes::SlotHash,
        system_program, sysvar,
    },
};
use anchor_spl::{
    associated_token,
    token::{self, Mint, MintTo, TokenAccount},
};

// TODO Rename Metadata to Config?

// TODO Upgrade to token22
// TODO Use the confidential transfers extension.
// TODO Use the memo extension?

declare_id!("CeJShZEAzBLwtcLQvbZc7UT38e4nUTn63Za5UFyYYDTS");

/// The decimal precision of the ORE token.
/// Using SI prefixes, the smallest indivisible unit of ORE is a nanoORE.
/// 1 nanoORE = 0.000000001 ORE = one billionth of an ORE
pub const TOKEN_DECIMALS: u8 = 9;

/// The mint address of the ORE token.
pub const TOKEN_MINT_ADDRESS: Pubkey = Pubkey::new_from_array([
    104, 116, 55, 208, 161, 233, 115, 227, 49, 65, 34, 153, 138, 61, 159, 228, 16, 158, 53, 8, 4,
    132, 86, 10, 198, 221, 80, 15, 115, 222, 47, 191,
]);

/// One ORE token, denominated in units of nanoORE.
pub const ONE_ORE: u64 = 10u64.pow(TOKEN_DECIMALS as u32);

/// The duration of an epoch, in units of seconds.
pub const EPOCH_DURATION: i64 = 60;

/// The target quantity of ORE to be mined per epoch, in units of nanoORE.
/// Inflation rate ≈ 1 ORE / epoch (min 0, max 2)
pub const TARGET_EPOCH_REWARDS: u64 = ONE_ORE;

/// The smoothing factor for reward rate changes. The reward rate cannot change by more or less
/// than factor of this constant from one epoch to the next.
pub const SMOOTHING_FACTOR: u64 = 2;

/// The number of bus accounts, for parallelizing mine operations.
pub const BUS_COUNT: u8 = 8;

/// The quantity of ORE each bus will be topped up with at the beginning of each epoch.
pub const BUS_BALANCE: u64 = TARGET_EPOCH_REWARDS
    .saturating_mul(SMOOTHING_FACTOR)
    .saturating_div(BUS_COUNT as u64);

/// The initial hashing difficulty. The admin authority can update this in the future, if needed.
pub const INITIAL_DIFFICULTY: Hash = Hash::new_from_array([
    0, 0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
]);

/// The initial reward rate to payout in the first epoch.
pub const INITIAL_REWARD_RATE: u64 = 10u64.pow(3u32);

// TODO Set this before deployment
/// The unix timestamp after which mining is allowed.
pub const START_AT: i64 = 0;

// Assert TARGET_EPOCH_REWARDS is evenly divisible by BUS_COUNT.
static_assertions::const_assert!(
    (TARGET_EPOCH_REWARDS / BUS_COUNT as u64) * BUS_COUNT as u64 == TARGET_EPOCH_REWARDS
);

#[program]
mod ore {
    use super::*;

    /// Initializes the metadata account. Can only be invoked once.
    pub fn initialize_metadata(ctx: Context<InitializeMetadata>) -> Result<()> {
        ctx.accounts.metadata.bump = ctx.bumps.metadata;
        ctx.accounts.metadata.admin = ctx.accounts.signer.key();
        ctx.accounts.metadata.difficulty = INITIAL_DIFFICULTY;
        ctx.accounts.metadata.reward_rate = INITIAL_REWARD_RATE;
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

    /// Initializes a proof account for a new miner. Can only invoked once per signer.
    pub fn initialize_proof(ctx: Context<InitializeProof>) -> Result<()> {
        ctx.accounts.proof.authority = ctx.accounts.signer.key();
        ctx.accounts.proof.bump = ctx.bumps.proof;
        ctx.accounts.proof.hash = hashv(&[&ctx.accounts.signer.key().to_bytes()]);
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
            .epoch_rewards
            .saturating_add(bus_1.epoch_rewards)
            .saturating_add(bus_2.epoch_rewards)
            .saturating_add(bus_3.epoch_rewards)
            .saturating_add(bus_4.epoch_rewards)
            .saturating_add(bus_5.epoch_rewards)
            .saturating_add(bus_6.epoch_rewards)
            .saturating_add(bus_7.epoch_rewards);

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

    /// Distributes Ore tokens to the signer if a valid hash is provided.
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
            metadata.difficulty,
        )?;

        // Update bus stats.
        let bus = &mut ctx.accounts.bus;
        bus.epoch_rewards = bus.epoch_rewards.saturating_add(metadata.reward_rate);

        // Update miner stats.
        proof.total_hashes = proof.total_hashes.saturating_add(1);
        proof.total_rewards = proof.total_rewards.saturating_add(1);

        // Hash a recent slot hash into the next hash to prevent pre-mining attacks.
        let slot_hash_bytes = &ctx.accounts.slot_hashes.data.borrow()[0..size_of::<SlotHash>()];
        let slot_hash: SlotHash =
            bincode::deserialize(slot_hash_bytes).expect("Failed to deserialize slot hash");
        proof.hash = hashv(&[hash.as_ref(), slot_hash.1.as_ref()]);

        // Distribute tokens from bus to beneficiary.
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

    /// Updates the admin to a new value. Can only be invoked by the admin authority.
    pub fn update_admin(ctx: Context<UpdateDifficulty>, new_admin: Pubkey) -> Result<()> {
        ctx.accounts.metadata.admin = new_admin;
        Ok(())
    }

    /// Updates the difficulty to a new value. Can only be invoked by the admin authority.
    ///
    /// Ore subdivides into 1B units of indivisible nanoORE. If global hashpower increases to the
    /// point where >1B valid hashes are being submitted per epoch, the Ore inflation rate could
    /// be pushed steadily above 1 ORE/epoch. The protocol guarantees inflation can never exceed
    /// 2 ORE/epoch, but it is the responsibility of the admin to adjust the mining difficulty
    /// as needed to maintain the 1 ORE/epoch average.
    ///
    /// It is worth noting that Solana today processes well below 1M real TPS or
    /// (60 * 1,000,000) = 60,000,000 hashes per epoch. Even if every transaction on the network
    /// were mine operation, this is still two orders of magnitude below the threshold where the
    /// Ore inflation rate would be challenged. So in practice, Solana is likely to reach its
    /// network saturation point long before the Ore inflation hits its boundary condition.
    pub fn update_difficulty(ctx: Context<UpdateDifficulty>, new_difficulty: Hash) -> Result<()> {
        ctx.accounts.metadata.difficulty = new_difficulty;
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
    let hash_ = hashv(&[
        current_hash.as_ref(),
        signer.as_ref(),
        nonce.to_be_bytes().as_slice(),
    ]);
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
        .saturating_mul(TARGET_EPOCH_REWARDS as u128)
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
    bus.epoch_rewards = 0;

    // Top up bus account.
    let amount = BUS_BALANCE.saturating_sub(bus_tokens.amount);
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

    /// The count of rewards issued by this bus in the current epoch.
    pub epoch_rewards: u64,
}

/// Metadata is an account type used to track global program variables.
#[account]
#[derive(Debug)]
pub struct Metadata {
    /// The bump of the metadata account PDA.
    pub bump: u8,

    /// The admin authority with permission to update the difficulty.
    pub admin: Pubkey,

    /// The hash difficulty.
    pub difficulty: Hash,

    /// The timestamp of the start of the current epoch.
    pub epoch_start_at: i64,

    /// The reward rate to payout to miners for submiting valid hashes.
    pub reward_rate: u64,
}

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

    /// The total lifetime hashes provided by this miner.
    pub total_hashes: u64,

    /// The total lifetime rewards distributed to this miner.
    pub total_rewards: u64,
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
    #[account(init, address = TOKEN_MINT_ADDRESS, payer = signer, mint::decimals = TOKEN_DECIMALS, mint::authority = metadata)]
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
    #[account(seeds = [METADATA], bump = metadata.bump)]
    pub metadata: Account<'info, Metadata>,

    /// The Ore token mint account.
    #[account(address = TOKEN_MINT_ADDRESS)]
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
    #[account(seeds = [METADATA], bump = metadata.bump)]
    pub metadata: Account<'info, Metadata>,

    /// The Ore token mint account.
    #[account(address = TOKEN_MINT_ADDRESS)]
    pub mint: Account<'info, Mint>,

    /// The bus account.
    #[account(constraint = bus.id.lt(&BUS_COUNT) @ ProgramError::BusInvalid)]
    pub bus: Account<'info, Bus>,

    /// The bus token account.
    #[account(init, associated_token::mint = mint, associated_token::authority = bus, payer = signer, constraint = bus.id.lt(&BUS_COUNT))]
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

/// InitializeProof initializes a new proof account for a miner.
#[derive(Accounts)]
pub struct InitializeProof<'info> {
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
    #[account(mut, address = TOKEN_MINT_ADDRESS)]
    pub mint: Account<'info, Mint>,

    /// The metadata account.
    #[account(mut, seeds = [METADATA], bump = metadata.bump)]
    pub metadata: Account<'info, Metadata>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,

    /// The SPL associated token program.
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,
}

/// Mine distributes Ore to the beneficiary if the signer provides a valid hash.
#[derive(Accounts)]
#[instruction(hash: Hash, nonce: u64)]
pub struct Mine<'info> {
    /// The signer of the transaction (i.e. the miner).
    #[account(mut, address = proof.authority)]
    pub signer: Signer<'info>,

    /// The beneficiary token account to mint rewards to.
    #[account(mut, token::mint = mint)]
    pub beneficiary: Account<'info, TokenAccount>,

    /// A bus account.
    #[account(mut, constraint = bus.id.lt(&BUS_COUNT) @ ProgramError::BusInvalid)]
    pub bus: Account<'info, Bus>,

    /// The bus' token account.
    #[account(mut, associated_token::mint = mint, associated_token::authority = bus)]
    pub bus_tokens: Account<'info, TokenAccount>,

    /// The metadata account.
    #[account(seeds = [METADATA], bump = metadata.bump)]
    pub metadata: Account<'info, Metadata>,

    /// The proof account.
    #[account(mut, seeds = [PROOF, signer.key().as_ref()], bump = proof.bump)]
    pub proof: Account<'info, Proof>,

    /// The Ore token mint account.
    #[account(address = TOKEN_MINT_ADDRESS)]
    pub mint: Account<'info, Mint>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,

    /// The slot hashes sysvar account.
    /// CHECK: SlotHashes is too large to deserialize. Instead we manually verify the sysvar address and deserialize only the slice we need.
    #[account(address = sysvar::slot_hashes::ID)]
    pub slot_hashes: AccountInfo<'info>,
}

/// UpdateAdmin allows the admin to reassign the admin authority.
#[derive(Accounts)]
#[instruction(new_admin: Pubkey)]
pub struct UpdateAdmin<'info> {
    /// The signer of the transaction (i.e. the admin).
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The metadata account.
    #[account(seeds = [METADATA], bump = metadata.bump, constraint = metadata.admin.eq(&signer.key()) @ ProgramError::NotAuthorized)]
    pub metadata: Account<'info, Metadata>,
}

/// UpdateDifficulty allows the admin to update the mining difficulty.
#[derive(Accounts)]
#[instruction(new_difficulty: Hash)]
pub struct UpdateDifficulty<'info> {
    /// The signer of the transaction (i.e. the admin).
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The metadata account.
    #[account(seeds = [METADATA], bump = metadata.bump, constraint = metadata.admin.eq(&signer.key()) @ ProgramError::NotAuthorized)]
    pub metadata: Account<'info, Metadata>,
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

    /// The current Ore token supply.
    pub supply: u64,

    /// The current mining difficulty.
    pub difficulty: Hash,

    /// The valid hash provided by the signer.
    pub hash: Hash,

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
    #[msg("An invalid bus account was provided")]
    BusInvalid,
    #[msg("This bus does not have enough tokens to pay the reward")]
    BusInsufficientFunds,
    #[msg("The signer is not authorized to perform this action")]
    NotAuthorized,
}

#[cfg(test)]
mod tests {
    use anchor_lang::{
        prelude::Pubkey,
        solana_program::hash::{hashv, Hash},
    };

    use crate::{calculate_new_reward_rate, validate_hash, SMOOTHING_FACTOR, TARGET_EPOCH_REWARDS};

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
        let new_rate = calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS);
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
        let new_rate =
            calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS.saturating_add(1_000_000));
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_higher() {
        let current_rate = 1000;
        let new_rate =
            calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS.saturating_sub(1_000_000));
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
