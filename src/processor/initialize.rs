use std::mem::size_of;

use solana_program::program_pack::Pack;
use solana_program::{self, sysvar};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};
use spl_token::state::Mint;

use crate::{instruction::*, BUS, INITIAL_DIFFICULTY, MINT_ADDRESS};
use crate::{
    loaders::*,
    state::{Bus, Treasury},
    utils::create_pda,
    BUS_COUNT, INITIAL_REWARD_RATE, MINT, TOKEN_DECIMALS, TREASURY,
};

pub fn process_initialize<'a, 'info>(
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
    let busses = vec![
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[0], &[args.bus_0_bump]],
        )?,
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[1], &[args.bus_1_bump]],
        )?,
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[2], &[args.bus_2_bump]],
        )?,
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[3], &[args.bus_3_bump]],
        )?,
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[4], &[args.bus_4_bump]],
        )?,
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[5], &[args.bus_5_bump]],
        )?,
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[6], &[args.bus_6_bump]],
        )?,
        load_uninitialized_pda(
            next_account_info(accounts_iter)?,
            &[BUS, &[7], &[args.bus_7_bump]],
        )?,
    ];

    // Account 10: Mint
    let mint = load_uninitialized_pda(
        next_account_info(accounts_iter)?,
        &[MINT, &[args.mint_bump]],
    )?;
    if !mint.key.eq(&MINT_ADDRESS) {
        return Err(ProgramError::InvalidAccountData);
    }

    // Account 11: Treasury
    let treasury_account_info = load_uninitialized_pda(
        next_account_info(accounts_iter)?,
        &[TREASURY, &[args.treasury_bump]],
    )?;

    // Account 12: Treasury tokens
    let treasury_tokens = load_uninitialized_account(next_account_info(accounts_iter)?)?;

    // Account 13: System program
    let system_program = load_account(next_account_info(accounts_iter)?, system_program::id())?;

    // Account 14: Token program
    let token_program = load_account(next_account_info(accounts_iter)?, spl_token::id())?;

    // Account 15: Associated token program
    let associated_token_program = load_account(
        next_account_info(accounts_iter)?,
        spl_associated_token_account::id(),
    )?;

    // Account 16: Rent sysvar
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
    treasury.difficulty = INITIAL_DIFFICULTY.into();
    treasury.reward_rate = INITIAL_REWARD_RATE;
    treasury.total_claimed_rewards = 0;
    drop(treasury_data);

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

    // Initialize treasury token account
    solana_program::program::invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            signer.key,
            treasury_account_info.key,
            mint.key,
            &spl_token::id(),
        ),
        &[
            associated_token_program.clone(),
            signer.clone(),
            treasury_tokens.clone(),
            treasury_account_info.clone(),
            mint.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}
