use std::mem::size_of;

use coal_api::{
    consts::*, loaders::{load_program, load_signer, load_sysvar}, 
    state::{Bus, WoodConfig}
};
use solana_program::{
    self, account_info::AccountInfo, entrypoint::ProgramResult, msg, 
    program_error::ProgramError, rent::Rent, system_program, sysvar::{self, Sysvar}
};

use crate::utils::{AccountDeserialize, Discriminator};

pub fn process_patch_wood<'a, 'info>(accounts: &'a [AccountInfo<'info>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [payer, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, system_program, rent_sysvar] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    load_signer(payer)?;
    load_program(system_program, system_program::id())?;
    load_sysvar(rent_sysvar, sysvar::rent::id())?;



    ensure_rent_exempt(config_info, 8 + size_of::<WoodConfig>(), system_program, payer)?;
    let config = WoodConfig {
        base_reward_rate: INITIAL_BASE_WOOD_REWARD_RATE,
        last_reset_at: 0,
        min_difficulty: INITIAL_MIN_DIFFICULTY as u64,
        top_balance: 0,
        total_epoch_rewards: INITIAL_WOOD_EPOCH_REWARDS,
    };
    let patched_config = WoodConfig::to_bytes(&config);
    // Ensure the account has enough space for the new config data
    let required_size = 8 + patched_config.len();
    config_info.realloc(required_size, true)?;
    let mut config_data = config_info.try_borrow_mut_data()?;
    config_data[0] = WoodConfig::discriminator() as u8;
    config_data[1..8].copy_from_slice(&[0; 7]); // Padding
    config_data[8..8 + patched_config.len()].copy_from_slice(&patched_config);

    msg!("Checking is config valid...");
    let patched_config = WoodConfig::try_from_bytes_mut(&mut config_data)?;
    msg!("Patched config: {:?}", patched_config);

    let bus_infos = [
        bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info,
        bus_7_info,
    ];

    for i in 0..BUS_COUNT {
        ensure_rent_exempt(bus_infos[i], 8 + size_of::<Bus>(), system_program, payer)?;
        let bus = Bus {
            id: i as u64,
            rewards: INITIAL_WOOD_EPOCH_REWARDS,
            theoretical_rewards: 0,
            top_balance: 0,      
        };
        // Ensure the account has enough space for the new bus data
        let patched_bus = Bus::to_bytes(&bus);
        let required_size = 8 + patched_bus.len();
        bus_infos[i].realloc(required_size, true)?;
        let mut bus_data = bus_infos[i].try_borrow_mut_data()?;
        bus_data[0] = Bus::discriminator() as u8;
        bus_data[1..8].copy_from_slice(&[0; 7]); // Padding
        bus_data[8..8 + patched_bus.len()].copy_from_slice(&patched_bus);
    }

    Ok(())
}

pub fn ensure_rent_exempt<'a, 'info>(
    account: &'a AccountInfo<'info>,
    space: usize,
    system_program: &'a AccountInfo<'info>,
    payer: &AccountInfo<'info>,
) -> ProgramResult {
    let rent = Rent::get()?;
    let rent_exempt_balance = rent
            .minimum_balance(space)
            .saturating_sub(account.lamports());

    if rent_exempt_balance.gt(&0) {
    solana_program::program::invoke(
        &solana_program::system_instruction::transfer(
            payer.key,
            account.key,
            rent_exempt_balance,
        ),
        &[
            payer.clone(),
            account.clone(),
            system_program.clone(),
        ],
        )?;
    }

    Ok(())
}