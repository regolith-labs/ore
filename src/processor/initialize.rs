use std::mem::size_of;

use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_program, {self, sysvar},
};
use spl_token::state::Mint;

use crate::{
    instruction::*,
    loaders::*,
    state::{Bus, Treasury},
    utils::create_pda,
    utils::AccountDeserialize,
    utils::Discriminator,
    BUS, BUS_COUNT, INITIAL_DIFFICULTY, INITIAL_REWARD_RATE, MINT, MINT_ADDRESS, TOKEN_DECIMALS,
    TREASURY, TREASURY_ADDRESS,
};

pub fn process_initialize<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = InitializeArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, mint_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program, rent_sysvar] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    // TODO Verify bus keys
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(bus_0_info, &[BUS, &[0], &[args.bus_0_bump]])?;
    load_uninitialized_pda(mint_info, &[MINT, &[args.mint_bump]])?;
    if !mint_info.key.eq(&MINT_ADDRESS) {
        return Err(ProgramError::InvalidAccountData);
    }
    load_uninitialized_pda(treasury_info, &[TREASURY, &[args.treasury_bump]])?;
    if !treasury_info.key.eq(&TREASURY_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }
    load_uninitialized_account(treasury_tokens_info)?;
    load_program(system_program, system_program::id())?;
    load_program(token_program, spl_token::id())?;
    load_program(associated_token_program, spl_associated_token_account::id())?;
    load_sysvar(rent_sysvar, sysvar::rent::id())?;

    // Initialize bus accounts
    let bus_infos = [
        bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info,
        bus_7_info,
    ];
    let bus_bumps = [
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
            bus_infos[i],
            &crate::id(),
            8 + size_of::<Bus>(),
            &[BUS, &[i as u8], &[bus_bumps[i]]],
            system_program,
            signer,
        )?;
        let mut bus_data = bus_infos[i].try_borrow_mut_data()?;
        bus_data[0] = Bus::discriminator() as u8;
        let mut bus = Bus::try_from_bytes_mut(&mut bus_data)?;
        bus.id = i as u64;
        bus.available_rewards = 0;
    }

    // Initialize treasury
    create_pda(
        treasury_info,
        &crate::id(),
        8 + size_of::<Treasury>(),
        &[TREASURY, &[args.treasury_bump]],
        system_program,
        signer,
    )?;
    let mut treasury_data = treasury_info.data.borrow_mut();
    treasury_data[0] = Treasury::discriminator() as u8;
    let mut treasury = Treasury::try_from_bytes_mut(&mut treasury_data)?;
    treasury.bump = args.treasury_bump as u64;
    treasury.admin = *signer.key;
    treasury.epoch_start_at = 0;
    treasury.difficulty = INITIAL_DIFFICULTY.into();
    treasury.reward_rate = INITIAL_REWARD_RATE;
    treasury.total_claimed_rewards = 0;
    drop(treasury_data);

    // Initialize mint
    create_pda(
        mint_info,
        &spl_token::id(),
        Mint::LEN,
        &[MINT, &[args.mint_bump]],
        system_program,
        signer,
    )?;
    solana_program::program::invoke_signed(
        &spl_token::instruction::initialize_mint(
            &spl_token::id(),
            mint_info.key,
            treasury_info.key,
            None,
            TOKEN_DECIMALS,
        )?,
        &[
            token_program.clone(),
            mint_info.clone(),
            treasury_info.clone(),
            rent_sysvar.clone(),
        ],
        &[&[MINT, &[args.mint_bump]]],
    )?;

    // Initialize treasury token account
    solana_program::program::invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            signer.key,
            treasury_info.key,
            mint_info.key,
            &spl_token::id(),
        ),
        &[
            associated_token_program.clone(),
            signer.clone(),
            treasury_tokens_info.clone(),
            treasury_info.clone(),
            mint_info.clone(),
            system_program.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}
