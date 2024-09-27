use ore_api::{
    consts::*,
    instruction::*,
    state::{Bus, Config, Treasury},
};
use solana_program::{
    self, account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    program_pack::Pack, system_program, sysvar,
};
use spl_token::state::Mint;
use steel::*;

/// Initialize sets up the ORE program to begin mining.
pub fn process_initialize(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse args.
    let args = Initialize::try_from_bytes(data)?;

    // Load accounts.
    let [signer_info, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, metadata_info, mint_info, treasury_info, treasury_tokens_info, system_program, token_program, associated_token_program, metadata_program, rent_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&INITIALIZER_ADDRESS)?;
    bus_0_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[0]], args.bus_0_bump, &ore_api::ID)?;
    bus_1_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[1]], args.bus_1_bump, &ore_api::ID)?;
    bus_2_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[2]], args.bus_2_bump, &ore_api::ID)?;
    bus_3_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[3]], args.bus_3_bump, &ore_api::ID)?;
    bus_4_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[4]], args.bus_4_bump, &ore_api::ID)?;
    bus_5_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[5]], args.bus_5_bump, &ore_api::ID)?;
    bus_6_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[6]], args.bus_6_bump, &ore_api::ID)?;
    bus_7_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[BUS, &[7]], args.bus_7_bump, &ore_api::ID)?;
    config_info
        .is_empty()?
        .is_writable()?
        .has_seeds(&[CONFIG], args.config_bump, &ore_api::ID)?;
    metadata_info.is_empty()?.is_writable()?.has_seeds(
        &[
            METADATA,
            mpl_token_metadata::ID.as_ref(),
            MINT_ADDRESS.as_ref(),
        ],
        args.metadata_bump,
        &mpl_token_metadata::ID,
    )?;
    mint_info.is_empty()?.is_writable()?.has_seeds(
        &[MINT, MINT_NOISE.as_slice()],
        args.mint_bump,
        &ore_api::ID,
    )?;
    treasury_info.is_empty()?.is_writable()?.has_seeds(
        &[TREASURY],
        args.treasury_bump,
        &ore_api::ID,
    )?;
    treasury_tokens_info.is_empty()?.is_writable()?;
    system_program.is_program(&system_program::ID)?;
    token_program.is_program(&spl_token::ID)?;
    associated_token_program.is_program(&spl_associated_token_account::ID)?;
    metadata_program.is_program(&mpl_token_metadata::ID)?;
    rent_sysvar.is_sysvar(&sysvar::rent::ID)?;

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
        create_account::<Bus>(
            bus_infos[i],
            &ore_api::id(),
            &[BUS, &[i as u8], &[bus_bumps[i]]],
            system_program,
            signer_info,
        )?;
        let bus = bus_infos[i].to_account_mut::<Bus>(&ore_api::ID)?;
        bus.id = i as u64;
        bus.rewards = 0;
        bus.theoretical_rewards = 0;
        bus.top_balance = 0;
    }

    // Initialize config.
    create_account::<Config>(
        config_info,
        &ore_api::id(),
        &[CONFIG, &[args.config_bump]],
        system_program,
        signer_info,
    )?;
    let config = config_info.to_account_mut::<Config>(&ore_api::ID)?;
    config.base_reward_rate = INITIAL_BASE_REWARD_RATE;
    config.last_reset_at = 0;
    config.min_difficulty = INITIAL_MIN_DIFFICULTY as u64;
    config.top_balance = 0;

    // Initialize treasury.
    create_account::<Treasury>(
        treasury_info,
        &ore_api::id(),
        &[TREASURY, &[args.treasury_bump]],
        system_program,
        signer_info,
    )?;

    // Initialize mint.
    allocate_account(
        mint_info,
        &spl_token::id(),
        Mint::LEN,
        &[MINT, MINT_NOISE.as_slice(), &[args.mint_bump]],
        system_program,
        signer_info,
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
        &[&[MINT, MINT_NOISE.as_slice(), &[args.mint_bump]]],
    )?;

    // Initialize mint metadata.
    mpl_token_metadata::instructions::CreateMetadataAccountV3Cpi {
        __program: metadata_program,
        metadata: metadata_info,
        mint: mint_info,
        mint_authority: treasury_info,
        payer: signer_info,
        update_authority: (signer_info, true),
        system_program,
        rent: Some(rent_sysvar),
        __args: mpl_token_metadata::instructions::CreateMetadataAccountV3InstructionArgs {
            data: mpl_token_metadata::types::DataV2 {
                name: METADATA_NAME.to_string(),
                symbol: METADATA_SYMBOL.to_string(),
                uri: METADATA_URI.to_string(),
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
    create_associated_token_account(
        signer_info,
        treasury_info,
        treasury_tokens_info,
        mint_info,
        system_program,
        token_program,
        associated_token_program,
    )?;

    Ok(())
}
