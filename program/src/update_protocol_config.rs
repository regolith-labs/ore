use ore_api::prelude::*;
use steel::*;

/// Sets the admin.
pub fn process_update_protocol_config(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = UpdateProtocolConfig::try_from_bytes(data)?;
    // let new_admin = Pubkey::new_from_array(args.admin);
    // let new_fee_collector = Pubkey::new_from_array(args.fee_collector);
    // let new_fee_rate = u64::from_le_bytes(args.fee_rate);
    let new_intermission_slots = u64::from_le_bytes(args.intermission_slots);
    let new_round_slots = u64::from_le_bytes(args.round_slots);
    // let new_entropy_var_address = Pubkey::new_from_array(args.entropy_var_address);
    // let new_entropy_program_id = Pubkey::new_from_array(args.entropy_program_id);

    // Load accounts.
    let [signer_info, config_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?.has_address(&ADMIN_ADDRESS)?;
    let config = config_info
        .has_seeds(&[CONFIG], &ore_api::ID)?
        .as_account_mut::<Config>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    // Set protocol config.
    config.protocol.round_slots = new_round_slots;
    config.protocol.intermission_slots = new_intermission_slots;

    Ok(())
}
