use solana_program::{
    account_info::AccountInfo, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::{
    instruction::UpdateDifficultyArgs, loaders::*, state::Treasury, utils::AccountDeserialize,
};

/// UpdateDifficulty updates the program's global difficulty value. Its responsibilities include:
/// 1. Update the mining difficulty.
///
/// Safety requirements:
/// - Can only succeed if the signer is the program admin.
/// - Can only succeed if the provided treasury is valid.
///
/// Discussion:
/// - Ore subdivides into 1 billion indivisible atomic units. Therefore if global hashpower
///   were to increase to the point where >1B valid hashes were submitted to the protocol for
///   validation per epoch, the Ore inflation rate could be pushed above the 1 ORE / min target.
/// - The strict limits on bus reward counters guarantee inflation can never exceed 2 ORE / min,
///   but it is the responsibility of the admin to adjust mining difficulty if needed to maintain
///   the 1 ORE / min target average.
/// - It is worth noting that Solana today processes well below 1 million real TPS or
///   (60 * 1,000,000) = 60,000,000 transactions per minute. Even if every transaction on Solana
///   were a mine operation, this would still be two orders of magnitude below the boundary
///   condition where Ore inflation targets would be challenged. So in practice, Solana is likely
///   to reach its network saturation point long before Ore ever hits its theoretical limits.
pub fn process_update_difficulty<'a, 'info>(
    _program_id: &Pubkey,
    accounts: &'a [AccountInfo<'info>],
    data: &[u8],
) -> ProgramResult {
    // Parse args
    let args = UpdateDifficultyArgs::try_from_bytes(data)?;

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
    treasury.difficulty = args.new_difficulty;

    Ok(())
}
