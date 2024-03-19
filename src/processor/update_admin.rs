use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{instruction::UpdateAdminArgs, loaders::*, state::Treasury, utils::AccountDeserialize};

/// UpdateAdmin updates the program's admin account. Its responsibilities include:
/// 1. Update the treasury admin address.
///
/// Safety requirements:
/// - Can only succeed if the signer is the current program admin.
/// - Can only succeed if the provided treasury is valid.
///
/// Discussion:
/// - The admin authority has one lever of power: the ability to adjust the global
///   mining difficulty. If the difficulty is too easy, miners will find hashes very quickly
///   and the bottleneck for mining will shift from local compute to Solana bandwidth. In essence,
///   if the Ore token has value and difficulty is low, mining becomes an incentivized stress
///   test for the Solana network.
/// - At the same time, if difficulty is too hard, miners will have to wait a very long period
///   of time between finding valid hashes. This will bias rewards to well-resourced miners
///   with large compute operations. Keeping a low difficulty ensures casual miners can
///   consistently earn rewards and undercuts some of the advantage of larger players.
/// - Ultimately admin authority should be delegated to a governance mechanism – either
///   democratic or futarchic – to ensure difficulty is kept at a value that represents the
///   values and interests of the whole ecosystem.
pub fn process_update_admin<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = UpdateAdminArgs::try_from_bytes(data)?;

    // Load accounts
    let [signer, treasury_info] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    load_signer(signer)?;
    load_treasury(treasury_info, true)?;

    // Validate signer is admin
    let mut treasury_data = treasury_info.data.borrow_mut();
    let treasury = Treasury::try_from_bytes_mut(&mut treasury_data)?;
    if treasury.admin.ne(&signer.key) {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Update admin
    treasury.admin = args.new_admin;

    Ok(())
}
