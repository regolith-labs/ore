use std::mem::size_of;

use coal_api::{
    consts::*,
    instruction::*,
    loaders::*,
    state::{Bus, Config, Treasury},
};
use coal_utils::spl::create_ata;
use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    program_pack::Pack,
    system_program, {self, sysvar},
};
use spl_token::state::Mint;

use crate::utils::{create_pda, AccountDeserialize, Discriminator};

/// Initialize sets up the ORE program to begin mining.
pub fn process_init_coal<'a, 'info>(
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args.
    let args = InitializeArgs::try_from_bytes(data)?;

    // Load accounts.
    let [signer, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, metadata_info, mint_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program, metadata_program, rent_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    
    load_signer(signer)?;
    load_uninitialized_pda(bus_0_info, &[COAL_BUS, &[0]], args.bus_0_bump, &coal_api::id())?;
    load_uninitialized_pda(bus_1_info, &[COAL_BUS, &[1]], args.bus_1_bump, &coal_api::id())?;
    load_uninitialized_pda(bus_2_info, &[COAL_BUS, &[2]], args.bus_2_bump, &coal_api::id())?;
    load_uninitialized_pda(bus_3_info, &[COAL_BUS, &[3]], args.bus_3_bump, &coal_api::id())?;
    load_uninitialized_pda(bus_4_info, &[COAL_BUS, &[4]], args.bus_4_bump, &coal_api::id())?;
    load_uninitialized_pda(bus_5_info, &[COAL_BUS, &[5]], args.bus_5_bump, &coal_api::id())?;
    load_uninitialized_pda(bus_6_info, &[COAL_BUS, &[6]], args.bus_6_bump, &coal_api::id())?;
    load_uninitialized_pda(bus_7_info, &[COAL_BUS, &[7]], args.bus_7_bump, &coal_api::id())?;
    load_uninitialized_pda(config_info, &[COAL_CONFIG], args.config_bump, &coal_api::id())?;
    load_uninitialized_pda(
        metadata_info,
        &[
            METADATA,
            mpl_token_metadata::ID.as_ref(),
            COAL_MINT_ADDRESS.as_ref(),
        ],
        args.metadata_bump,
        &mpl_token_metadata::ID,
    )?;
    load_uninitialized_pda(
        mint_info,
        &[COAL_MINT, MINT_NOISE.as_slice()],
        args.mint_bump,
        &coal_api::id(),
    )?;
    load_uninitialized_pda(
        treasury_info,
        &[TREASURY],
        args.treasury_bump,
        &coal_api::id(),
    )?;
    load_system_account(treasury_tokens_info, true)?;
    load_program(system_program, system_program::id())?;
    load_program(token_program, spl_token::id())?;
    load_program(associated_token_program, spl_associated_token_account::id())?;
    load_program(metadata_program, mpl_token_metadata::ID)?;
    load_sysvar(rent_sysvar, sysvar::rent::id())?;

    // Check signer.
    if signer.key.ne(&INITIALIZER_ADDRESS) {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Initialize bus accounts.
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
            &coal_api::id(),
            8 + size_of::<Bus>(),
            &[COAL_BUS, &[i as u8], &[bus_bumps[i]]],
            system_program,
            signer,
        )?;
        let mut bus_data = bus_infos[i].try_borrow_mut_data()?;
        bus_data[0] = Bus::discriminator() as u8;
        let bus = Bus::try_from_bytes_mut(&mut bus_data)?;
        bus.id = i as u64;
        bus.rewards = 0;
        bus.theoretical_rewards = 0;
        bus.top_balance = 0;
    }

    // Initialize config.
    create_pda(
        config_info,
        &coal_api::id(),
        8 + size_of::<Config>(),
        &[COAL_CONFIG, &[args.config_bump]],
        system_program,
        signer,
    )?;
    let mut config_data = config_info.data.borrow_mut();
    config_data[0] = Config::discriminator() as u8;
    let config = Config::try_from_bytes_mut(&mut config_data)?;
    config.base_reward_rate = INITIAL_BASE_COAL_REWARD_RATE;
    config.last_reset_at = 0;
    config.min_difficulty = INITIAL_MIN_DIFFICULTY as u64;
    config.top_balance = 0;

    // Initialize treasury.
    create_pda(
        treasury_info,
        &coal_api::id(),
        8 + size_of::<Treasury>(),
        &[TREASURY, &[args.treasury_bump]],
        system_program,
        signer,
    )?;
    let mut treasury_data = treasury_info.data.borrow_mut();
    treasury_data[0] = Treasury::discriminator() as u8;
    drop(treasury_data);

    // Initialize mint.
    create_pda(
        mint_info,
        &spl_token::id(),
        Mint::LEN,
        &[COAL_MINT, MINT_NOISE.as_slice(), &[args.mint_bump]],
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
        &[&[COAL_MINT, MINT_NOISE.as_slice(), &[args.mint_bump]]],
    )?;

    // Initialize mint metadata.
    mpl_token_metadata::instructions::CreateMetadataAccountV3Cpi {
        __program: metadata_program,
        metadata: metadata_info,
        mint: mint_info,
        mint_authority: treasury_info,
        payer: signer,
        update_authority: (signer, true),
        system_program,
        rent: Some(rent_sysvar),
        __args: mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs {
            data: mpl_token_metadata::types::DataV2 {
                name: COAL_METADATA_NAME.to_string(),
                symbol: COAL_METADATA_SYMBOL.to_string(),
                uri: COAL_METADATA_URI.to_string(),
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            },
            is_mutable: true,
            collection_details: None,
        },
    }
    .invoke_signed(&[&[TREASURY, &[args.treasury_bump]]])?;

    // Initialize treasury token account.
    create_ata(
        signer,
        treasury_info,
        treasury_tokens_info,
        mint_info,
        system_program,
        token_program,
        associated_token_program,
    )?;

    Ok(())
}
