pub mod instruction;
mod loaders;

use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use solana_program::program_pack::Pack;
use solana_program::{self, sysvar};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    declare_id,
    entrypoint::ProgramResult,
    keccak::{hashv, Hash},
    program_error::ProgramError,
    pubkey,
    pubkey::Pubkey,
    rent::Rent,
    system_program,
    sysvar::Sysvar,
};

use instruction::*;
use loaders::*;
use spl_token::state::Mint;

// TODO Test admin and difficulty adjustment functions
// TODO Use more decimals?

declare_id!("CeJShZEAzBLwtcLQvbZc7UT38e4nUTn63Za5UFyYYDTS");

#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

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
pub const MINT_ADDRESS: Pubkey = pubkey!("37TDfMS8NHpyhyCXBrY9m7rRrtj1f7TrFzD1iXqmTeUX");

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

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    data: &[u8],
) -> ProgramResult {
    let (tag, data) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;

    let ix = OreInstruction::try_from(*tag).or(Err(ProgramError::InvalidInstructionData))?;

    match ix {
        OreInstruction::Epoch => process_epoch(program_id, accounts, data)?,
        OreInstruction::Proof => process_proof(program_id, accounts, data)?,
        OreInstruction::Mine => process_mine(program_id, accounts, data)?,
        OreInstruction::Claim => process_claim(program_id, accounts, data)?,
        OreInstruction::Initialize => process_initialize(program_id, accounts, data)?,
    }

    Ok(())
}

fn process_epoch<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // TODO
    Ok(())
}

fn process_proof<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // TODO
    Ok(())
}

fn process_mine<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // TODO
    Ok(())
}

fn process_claim<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // TODO
    Ok(())
}

fn process_initialize<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let args = bytemuck::try_from_bytes::<InitializeArgs>(data)
        .or(Err(ProgramError::InvalidInstructionData))?;

    // Account 1: Signer
    let signer = load_signer(next_account_info(accounts_iter)?)?;

    // Accounts 2-9: Busses
    #[rustfmt::skip]
    let busses = vec![
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[0], &[args.bus_0_bump]])?,
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[1], &[args.bus_1_bump]])?,
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[2], &[args.bus_2_bump]])?,
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[3], &[args.bus_3_bump]])?,
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[4], &[args.bus_4_bump]])?,
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[5], &[args.bus_5_bump]])?,
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[6], &[args.bus_6_bump]])?,
        load_uninitialized_pda(next_account_info(accounts_iter)?, &[BUS, &[7], &[args.bus_7_bump]])?,
    ];

    // Account 10: Mint
    #[rustfmt::skip]
    let mint = load_uninitialized_pda(next_account_info(accounts_iter)?, &[MINT, &[args.mint_bump]])?;

    // Account 11: Treasury
    #[rustfmt::skip]
    let treasury_account_info = load_uninitialized_pda(next_account_info(accounts_iter)?, &[TREASURY, &[args.treasury_bump]])?;

    // Account 12: Treasury tokens
    let treasury_tokens = load_uninitialized_account(next_account_info(accounts_iter)?)?;

    // Account 13: System program
    let system_program = load_account(next_account_info(accounts_iter)?, system_program::id())?;

    // Account 14: Token program
    let token_program = load_account(next_account_info(accounts_iter)?, spl_token::id())?;

    // Account 15: Rent sysvar
    let rent_sysvar = load_account(next_account_info(accounts_iter)?, sysvar::rent::id())?;

    // Initialize bus accounts
    let bus_bumps = vec![
        args.bus_0_bump,
        args.bus_1_bump,
        args.bus_2_bump,
        args.bus_3_bump,
        args.bus_4_bump,
        args.bus_5_bump,
        args.bus_6_bump,
        args.bus_7_bump,
    ];
    for i in 0..BUS_COUNT {
        create_pda(
            busses[i],
            &crate::id(),
            size_of::<Bus>(),
            &[BUS, &[i as u8], &[bus_bumps[i]]],
            system_program,
            signer,
        )?;
        busses[i].try_borrow_mut_data()?.copy_from_slice(
            Bus {
                bump: bus_bumps[i] as u32,
                id: i as u32,
                available_rewards: 0,
            }
            .to_bytes(),
        );
    }

    // Initialize treasury
    create_pda(
        treasury_account_info,
        &crate::id(),
        size_of::<Treasury>(),
        &[TREASURY, &[args.treasury_bump]],
        system_program,
        signer,
    )?;
    let mut treasury_data = treasury_account_info.data.borrow_mut();
    let mut treasury = bytemuck::try_from_bytes_mut::<Treasury>(&mut treasury_data).unwrap();
    treasury.bump = args.treasury_bump as u64;
    treasury.admin = *signer.key;
    treasury.epoch_start_at = 0;
    treasury.reward_rate = INITIAL_REWARD_RATE;
    treasury.total_claimed_rewards = 0;

    // Initialize mint
    create_pda(
        mint,
        &spl_token::id(),
        Mint::LEN,
        &[MINT, &[args.mint_bump]],
        system_program,
        signer,
    )?;
    solana_program::program::invoke_signed(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint.key,
            treasury_account_info.key,
            None,
            TOKEN_DECIMALS,
        )?,
        &[
            token_program.clone(),
            mint.clone(),
            treasury_account_info.clone(),
            rent_sysvar.clone(),
        ],
        &[&[MINT, &[args.mint_bump]]],
    )?;

    // TODO Initialize treasury token account
    create_pda(
        mint,
        &spl_token::id(),
        spl_token::state::Account::LEN,
        &[MINT, &[args.mint_bump]],
        system_program,
        signer,
    )?;
    solana_program::program::invoke_signed(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint.key,
            treasury_account_info.key,
            None,
            TOKEN_DECIMALS,
        )?,
        &[
            token_program.clone(),
            mint.clone(),
            treasury_account_info.clone(),
            rent_sysvar.clone(),
        ],
        &[&[MINT, &[args.mint_bump]]],
    )?;

    Ok(())
}

fn validate_hash(
    current_hash: Hash,
    hash: Hash,
    signer: Pubkey,
    nonce: u64,
    difficulty: Hash,
) -> Result<(), ProgramError> {
    // Validate hash correctness.
    let hash_ = hashv(&[
        current_hash.as_ref(),
        signer.as_ref(),
        nonce.to_be_bytes().as_slice(),
    ]);
    if !hash.eq(&hash_) {
        return Err(ProgramError::Custom(1));
    }

    // Validate hash difficulty.
    if !hash.le(&difficulty) {
        return Err(ProgramError::Custom(1));
    }

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

/// Creates a new pda
#[inline(always)]
pub fn create_pda<'a, 'info>(
    target_account: &'a AccountInfo<'info>,
    owner: &Pubkey,
    space: usize,
    pda_seeds: &[&[u8]],
    system_program: &'a AccountInfo<'info>,
    payer: &'a AccountInfo<'info>,
) -> ProgramResult {
    let rent = Rent::get()?;
    solana_program::program::invoke_signed(
        &solana_program::system_instruction::create_account(
            payer.key,
            target_account.key,
            rent.minimum_balance(space as usize),
            space as u64,
            owner,
        ),
        &[
            payer.clone(),
            target_account.clone(),
            system_program.clone(),
        ],
        &[pda_seeds],
    )?;
    Ok(())
}

/// The seed of the bus account PDA.
pub const BUS: &[u8] = b"bus";

/// The seed of the mint account PDA.
pub const MINT: &[u8] = b"mint";

/// The seed of the proof account PDA.
pub const PROOF: &[u8] = b"proof";

/// The seed of the treasury account PDA.
pub const TREASURY: &[u8] = b"treasury";

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Bus {
    /// The bump of the bus account PDA.
    pub bump: u32,

    /// The ID of the bus account.
    pub id: u32,

    /// The quantity of rewards this bus can issue in the current epoch epoch.
    pub available_rewards: u64,
}

impl Bus {
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Treasury {
    /// The bump of the treasury account PDA.
    pub bump: u64,

    /// The admin authority with permission to update the difficulty.
    pub admin: Pubkey,

    /// The hash difficulty.
    // pub difficulty: Hash,

    /// The timestamp of the start of the current epoch.
    pub epoch_start_at: i64,

    /// The reward rate to payout to miners for submiting valid hashes.
    pub reward_rate: u64,

    /// The total lifetime claimed rewards.
    pub total_claimed_rewards: u64,
}

impl Treasury {
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

#[cfg(test)]
mod tests {
    use solana_program::{
        keccak::{hashv, Hash},
        pubkey::Pubkey,
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
