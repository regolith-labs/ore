use std::mem::size_of;

use anchor_lang::{
    prelude::*,
    solana_program::{
        keccak::{hashv, Hash},
        slot_hashes::SlotHash,
        system_program, sysvar,
    },
};
use anchor_spl::{
    associated_token,
    token::{self, Mint, MintTo, TokenAccount},
};

// TODO Test admin and difficulty adjustment functions!

declare_id!("CeJShZEAzBLwtcLQvbZc7UT38e4nUTn63Za5UFyYYDTS");

// TODO Set this before deployment
/// The unix timestamp after which mining is allowed.
pub const START_AT: i64 = 0;

/// The initial reward rate to payout in the first epoch.
pub const INITIAL_REWARD_RATE: u64 = 10u64.pow(3u32);

/// The initial hashing difficulty. The admin authority can update this in the future, if needed.
pub const INITIAL_DIFFICULTY: Hash = Hash::new_from_array([
    0, 0, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
]);

/// The mint address of the ORE token.
pub const TOKEN_MINT_ADDRESS: Pubkey = Pubkey::new_from_array([
    31, 94, 128, 251, 8, 214, 16, 114, 78, 71, 1, 151, 221, 103, 239, 180, 136, 178, 202, 102, 159,
    185, 95, 250, 9, 18, 207, 100, 215, 105, 39, 64,
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
pub const BUS_COUNT: u8 = 8;

/// The smoothing factor for reward rate changes. The reward rate cannot change by more or less
/// than factor of this constant from one epoch to the next.
pub const SMOOTHING_FACTOR: u64 = 2;

// Assert MAX_EPOCH_REWARDS is evenly divisible by BUS_COUNT.
static_assertions::const_assert!(
    (MAX_EPOCH_REWARDS / BUS_COUNT as u64) * BUS_COUNT as u64 == MAX_EPOCH_REWARDS
);

#[program]
mod ore {
    use super::*;

    /// Initializes the treasury account. Can only be invoked once.
    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
        ctx.accounts.treasury.bump = ctx.bumps.treasury;
        ctx.accounts.treasury.admin = ctx.accounts.signer.key();
        ctx.accounts.treasury.difficulty = INITIAL_DIFFICULTY;
        ctx.accounts.treasury.reward_rate = INITIAL_REWARD_RATE;
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
        let treasury = &mut ctx.accounts.treasury;
        let epoch_end_at = treasury.epoch_start_at.saturating_add(EPOCH_DURATION);
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
        let total_available_rewards = bus_0
            .available_rewards
            .saturating_add(bus_1.available_rewards)
            .saturating_add(bus_2.available_rewards)
            .saturating_add(bus_3.available_rewards)
            .saturating_add(bus_4.available_rewards)
            .saturating_add(bus_5.available_rewards)
            .saturating_add(bus_6.available_rewards)
            .saturating_add(bus_7.available_rewards);
        let total_epoch_rewards = MAX_EPOCH_REWARDS.saturating_sub(total_available_rewards);

        // Update the reward amount for the next epoch.
        treasury.reward_rate = calculate_new_reward_rate(treasury.reward_rate, total_epoch_rewards);
        treasury.epoch_start_at = clock.unix_timestamp;

        // Reset bus accounts.
        bus_0.available_rewards = BUS_EPOCH_REWARDS;
        bus_1.available_rewards = BUS_EPOCH_REWARDS;
        bus_2.available_rewards = BUS_EPOCH_REWARDS;
        bus_3.available_rewards = BUS_EPOCH_REWARDS;
        bus_4.available_rewards = BUS_EPOCH_REWARDS;
        bus_5.available_rewards = BUS_EPOCH_REWARDS;
        bus_6.available_rewards = BUS_EPOCH_REWARDS;
        bus_7.available_rewards = BUS_EPOCH_REWARDS;

        // Top up treasury token account.
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: treasury.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.treasury_tokens.to_account_info(),
                },
                &[&[TREASURY, &[treasury.bump]]],
            ),
            total_epoch_rewards,
        )?;

        // Log data.
        msg!("Epoch rewards: {:?}", total_epoch_rewards);
        msg!("Reward rate: {:?}", treasury.reward_rate);
        msg!("Supply: {:?}", ctx.accounts.mint.supply);

        Ok(())
    }

    /// Distributes Ore tokens to the signer if a valid hash is provided.
    pub fn mine(ctx: Context<Mine>, hash: Hash, nonce: u64) -> Result<()> {
        // Validate epoch is active.
        let clock = Clock::get().unwrap();
        let treasury = &mut ctx.accounts.treasury;
        let epoch_end_at = treasury.epoch_start_at.saturating_add(EPOCH_DURATION);
        require!(
            clock.unix_timestamp.lt(&epoch_end_at),
            ProgramError::EpochNeedsReset
        );

        // Validate provided hash.
        let proof = &mut ctx.accounts.proof;
        validate_hash(
            proof.hash.clone(),
            hash.clone(),
            ctx.accounts.signer.key(),
            nonce,
            treasury.difficulty,
        )?;

        // Update claimable rewards.
        let bus = &mut ctx.accounts.bus;
        require!(
            bus.available_rewards.ge(&treasury.reward_rate),
            ProgramError::BusInsufficientFunds
        );
        bus.available_rewards = bus.available_rewards.saturating_sub(treasury.reward_rate);
        proof.claimable_rewards = proof.claimable_rewards.saturating_add(treasury.reward_rate);

        // Hash most recent slot hash into the next challenge to prevent pre-mining attacks.
        let slot_hash_bytes = &ctx.accounts.slot_hashes.data.borrow()[0..size_of::<SlotHash>()];
        let slot_hash: SlotHash = bincode::deserialize(slot_hash_bytes).unwrap();
        proof.hash = hashv(&[hash.as_ref(), slot_hash.1.as_ref()]);

        // Update lifetime stats.
        proof.total_hashes = proof.total_hashes.saturating_add(1);
        proof.total_rewards = proof.total_rewards.saturating_add(1);

        // Log data.
        msg!("Reward rate: {:?}", treasury.reward_rate);
        msg!("Claimable rewards: {:?}", proof.claimable_rewards);
        msg!("Total hashes: {:?}", proof.total_hashes);
        msg!("Total rewards: {:?}", proof.total_rewards);

        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, amount: u64) -> Result<()> {
        // Validate claim is for an appropriate quantity of tokens.
        let proof = &mut ctx.accounts.proof;
        require!(
            proof.claimable_rewards.ge(&amount),
            ProgramError::ClaimTooLarge
        );

        // Update claimable amount.
        proof.claimable_rewards = proof.claimable_rewards.saturating_sub(amount);

        // Update lifetime status.
        let treasury = &mut ctx.accounts.treasury;
        treasury.total_claimed_rewards = treasury.total_claimed_rewards.saturating_add(amount);

        // Distribute tokens from treasury to beneficiary.
        let treasury_tokens = &ctx.accounts.treasury_tokens;
        require!(
            treasury_tokens.amount.ge(&amount),
            ProgramError::TreasuryInsufficientFunds
        );
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: treasury_tokens.to_account_info(),
                    to: ctx.accounts.beneficiary.to_account_info(),
                    authority: treasury.to_account_info(),
                },
                &[&[TREASURY, &[treasury.bump]]],
            ),
            amount,
        )?;

        // Log data.
        msg!("Claimable rewards: {:?}", proof.claimable_rewards);
        msg!(
            "Total claimed rewards: {:?}",
            treasury.total_claimed_rewards
        );

        Ok(())
    }

    /// Updates the admin to a new value. Can only be invoked by the admin authority.
    pub fn update_admin(ctx: Context<UpdateDifficulty>, new_admin: Pubkey) -> Result<()> {
        ctx.accounts.treasury.admin = new_admin;
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
        ctx.accounts.treasury.difficulty = new_difficulty;
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

    // Prevent reward rate from dropping below 1 or exceeding BUS_EPOCH_REWARDS and return.
    new_rate_smoothed.max(1).min(BUS_EPOCH_REWARDS)
}

/// The seed of the bus account PDA.
pub const BUS: &[u8] = b"bus";

/// The seed of the proof account PDA.
pub const PROOF: &[u8] = b"proof";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

/// Bus is an account type used to track the number of processed hashes and issued rewards
/// during an epoch. There are 8 bus accounts to provide sufficient parallelism for mine ops
/// and reduce write lock contention.
#[account]
#[derive(Debug, PartialEq)]
pub struct Bus {
    /// The bump of the bus account PDA.
    pub bump: u8,

    /// The ID of the bus account.
    pub id: u8,

    /// The quantity of rewards this bus can issue in the current epoch epoch.
    pub available_rewards: u64,
}

/// Proof is an account type used to track a miner's hash chain.
#[account]
#[derive(Debug, PartialEq)]
pub struct Proof {
    /// The bump of the proof account PDA.
    pub bump: u8,

    /// The account (i.e. miner) authorized to use this proof.
    pub authority: Pubkey,

    /// The quantity of tokens this miner may claim from the treasury.
    pub claimable_rewards: u64,

    /// The proof's current hash.
    pub hash: Hash,

    /// The total lifetime hashes provided by this miner.
    pub total_hashes: u64,

    /// The total lifetime rewards distributed to this miner.
    pub total_rewards: u64,
}

/// Treasury is an account type used to track global program variables.
#[account]
#[derive(Debug, PartialEq)]
pub struct Treasury {
    /// The bump of the treasury account PDA.
    pub bump: u8,

    /// The admin authority with permission to update the difficulty.
    pub admin: Pubkey,

    /// The hash difficulty.
    pub difficulty: Hash,

    /// The timestamp of the start of the current epoch.
    pub epoch_start_at: i64,

    /// The reward rate to payout to miners for submiting valid hashes.
    pub reward_rate: u64,

    /// The total lifetime claimed rewards.
    pub total_claimed_rewards: u64,
}

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The Ore token mint.
    #[account(init, address = TOKEN_MINT_ADDRESS, payer = signer, mint::decimals = TOKEN_DECIMALS, mint::authority = treasury)]
    pub mint: Account<'info, Mint>,

    /// The treasury account.
    #[account(init, seeds = [TREASURY], bump, payer = signer, space = 8 + size_of::<Treasury>())]
    pub treasury: Account<'info, Treasury>,

    /// The treasury token account.
    #[account(init, associated_token::mint = mint, associated_token::authority = treasury, payer = signer)]
    pub treasury_tokens: Account<'info, TokenAccount>,

    /// The Solana system program.
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,

    /// The SPL associated token program.
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, associated_token::AssociatedToken>,

    /// The rent sysvar account.
    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializeBusses<'info> {
    /// The signer of the transaction.
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The treasury account.
    #[account(seeds = [TREASURY], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,

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

    /// The Ore token mint account.
    #[account(mut, address = TOKEN_MINT_ADDRESS)]
    pub mint: Account<'info, Mint>,

    /// The treasury account.
    #[account(mut, seeds = [TREASURY], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,

    /// The treasury token account.
    #[account(mut, associated_token::mint = mint, associated_token::authority = treasury)]
    pub treasury_tokens: Account<'info, TokenAccount>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,
}

/// Mine distributes Ore to the beneficiary if the signer provides a valid hash.
#[derive(Accounts)]
#[instruction(hash: Hash, nonce: u64)]
pub struct Mine<'info> {
    /// The signer of the transaction (i.e. the miner).
    #[account(mut, address = proof.authority)]
    pub signer: Signer<'info>,

    /// A bus account.
    #[account(mut, constraint = bus.id.lt(&BUS_COUNT) @ ProgramError::BusInvalid)]
    pub bus: Account<'info, Bus>,

    /// The proof account.
    #[account(mut, seeds = [PROOF, signer.key().as_ref()], bump = proof.bump)]
    pub proof: Account<'info, Proof>,

    /// The treasury account.
    #[account(seeds = [TREASURY], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,

    /// The slot hashes sysvar account.
    /// CHECK: SlotHashes is too large to deserialize. Instead we manually verify the sysvar address and deserialize only the slice we need.
    #[account(address = sysvar::slot_hashes::ID)]
    pub slot_hashes: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct Claim<'info> {
    /// The signer of the transaction (i.e. the miner).
    #[account(mut, address = proof.authority)]
    pub signer: Signer<'info>,

    /// The beneficiary token account to distribute rewards to.
    #[account(mut, token::mint = mint)]
    pub beneficiary: Account<'info, TokenAccount>,

    /// The Ore token mint account.
    #[account(address = TOKEN_MINT_ADDRESS)]
    pub mint: Account<'info, Mint>,

    /// The proof account.
    #[account(mut, seeds = [PROOF, signer.key().as_ref()], bump = proof.bump)]
    pub proof: Account<'info, Proof>,

    /// The treasury account.
    #[account(mut, seeds = [TREASURY], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,

    /// The treasury token account.
    #[account(mut, associated_token::mint = mint, associated_token::authority = treasury)]
    pub treasury_tokens: Account<'info, TokenAccount>,

    /// The SPL token program.
    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, token::Token>,
}

/// UpdateAdmin allows the admin to reassign the admin authority.
#[derive(Accounts)]
#[instruction(new_admin: Pubkey)]
pub struct UpdateAdmin<'info> {
    /// The signer of the transaction (i.e. the admin).
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The treasury account.
    #[account(mut, seeds = [TREASURY], bump = treasury.bump, constraint = treasury.admin.eq(&signer.key()) @ ProgramError::NotAuthorized)]
    pub treasury: Account<'info, Treasury>,
}

/// UpdateDifficulty allows the admin to update the mining difficulty.
#[derive(Accounts)]
#[instruction(new_difficulty: Hash)]
pub struct UpdateDifficulty<'info> {
    /// The signer of the transaction (i.e. the admin).
    #[account(mut)]
    pub signer: Signer<'info>,

    /// The treasury account.
    #[account(mut, seeds = [TREASURY], bump = treasury.bump, constraint = treasury.admin.eq(&signer.key()) @ ProgramError::NotAuthorized)]
    pub treasury: Account<'info, Treasury>,
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
    #[msg("You cannot claim more tokens than are available")]
    ClaimTooLarge,
    #[msg("The treasury does not have enough tokens to honor the claim")]
    TreasuryInsufficientFunds,
}

#[cfg(test)]
mod tests {
    use anchor_lang::{
        prelude::Pubkey,
        solana_program::keccak::{hashv, Hash},
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
