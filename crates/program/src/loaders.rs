use solana_program::{
    account_info::AccountInfo, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
    system_program, sysvar,
};
use spl_token::state::Mint;

use crate::{
    state::{Bus, Proof, Treasury},
    utils::AccountDeserialize,
    BUS_ADDRESSES, BUS_COUNT, MINT_ADDRESS, TREASURY_ADDRESS,
};

/// Errors if:
/// - Account is not a signer.
pub fn load_signer<'a, 'info>(info: &'a AccountInfo<'info>) -> Result<(), ProgramError> {
    if !info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Address does not match the expected bus address.
/// - Data is empty.
/// - Data cannot deserialize into a bus account.
/// - Bus ID does not match the expected ID.
/// - Expected to be writable, but is not.
pub fn load_bus<'a, 'info>(
    info: &'a AccountInfo<'info>,
    id: u64,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.key.ne(&BUS_ADDRESSES[id as usize]) {
        return Err(ProgramError::InvalidSeeds);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let bus_data = info.data.borrow();
    let bus = Bus::try_from_bytes(&bus_data)?;

    if bus.id.ne(&id) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Data is empty.
/// - Data cannot deserialize into a bus account.
/// - Bus ID is not in the expected range.
/// - Address is not in set of valid bus address.
/// - Expected to be writable, but is not.
pub fn load_any_bus<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let bus_data = info.data.borrow();
    let bus = Bus::try_from_bytes(&bus_data)?;

    if bus.id.ge(&(BUS_COUNT as u64)) {
        return Err(ProgramError::InvalidAccountData);
    }

    if info.key.ne(&BUS_ADDRESSES[bus.id as usize]) {
        return Err(ProgramError::InvalidSeeds);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Data is empty.
/// - Data cannot deserialize into a proof account.
/// - Proof authority does not match the expected address.
/// - Expected to be writable, but is not.
pub fn load_proof<'a, 'info>(
    info: &'a AccountInfo<'info>,
    authority: &Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let proof_data = info.data.borrow();
    let proof = Proof::try_from_bytes(&proof_data)?;

    if proof.authority.ne(&authority) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not Ore program.
/// - Address does not match the expected address.
/// - Data is empty.
/// - Data cannot deserialize into a treasury account.
/// - Expected to be writable, but is not.
pub fn load_treasury<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&crate::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.key.ne(&TREASURY_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let treasury_data = info.data.borrow();
    let _ = Treasury::try_from_bytes(&treasury_data)?;

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not SPL token program.
/// - Address does not match the expected mint address.
/// - Data is empty.
/// - Data cannot deserialize into a mint account.
/// - Expected to be writable, but is not.
pub fn load_mint<'a, 'info>(
    info: &'a AccountInfo<'info>,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.key.ne(&MINT_ADDRESS) {
        return Err(ProgramError::InvalidSeeds);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let mint_data = info.data.borrow();
    if Mint::unpack_unchecked(&mint_data).is_err() {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not SPL token program.
/// - Data is empty.
/// - Data cannot deserialize into a token account.
/// - Token account owner does not match the expected owner address.
/// - Token account mint does not match the expected mint address.
/// - Expected to be writable, but is not.
pub fn load_token_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
    owner: Option<&Pubkey>,
    mint: &Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.owner.ne(&spl_token::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if info.data_is_empty() {
        return Err(ProgramError::UninitializedAccount);
    }

    let account_data = info.data.borrow();
    let account = spl_token::state::Account::unpack_unchecked(&account_data)
        .or(Err(ProgramError::InvalidAccountData))?;

    if account.mint.ne(&mint) {
        return Err(ProgramError::InvalidAccountData);
    }

    if let Some(owner) = owner {
        if account.owner.ne(owner) {
            return Err(ProgramError::InvalidAccountData);
        }
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Address does not match PDA derived from provided seeds.
/// - Cannot load as an uninitialized account.
pub fn load_uninitialized_pda<'a, 'info>(
    info: &'a AccountInfo<'info>,
    seeds: &[&[u8]],
    bump: u8,
    program_id: &Pubkey,
) -> Result<(), ProgramError> {
    let pda = Pubkey::find_program_address(seeds, program_id);

    if info.key.ne(&pda.0) {
        return Err(ProgramError::InvalidSeeds);
    }

    if bump.ne(&pda.1) {
        return Err(ProgramError::InvalidSeeds);
    }

    load_uninitialized_account(info)
}

/// Errors if:
/// - Owner is not the system program.
/// - Data is not empty.
/// - Account is not writable.
pub fn load_uninitialized_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
) -> Result<(), ProgramError> {
    if info.owner.ne(&system_program::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    if !info.data_is_empty() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    if !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Owner is not the sysvar address.
/// - Account cannot load with the expected address.
pub fn load_sysvar<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<(), ProgramError> {
    if info.owner.ne(&sysvar::id()) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    load_account(info, key, false)
}

/// Errors if:
/// - Address does not match the expected value.
/// - Expected to be writable, but is not.
pub fn load_account<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
    is_writable: bool,
) -> Result<(), ProgramError> {
    if info.key.ne(&key) {
        return Err(ProgramError::InvalidAccountData);
    }

    if is_writable && !info.is_writable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

/// Errors if:
/// - Address does not match the expected value.
/// - Account is not executable.
pub fn load_program<'a, 'info>(
    info: &'a AccountInfo<'info>,
    key: Pubkey,
) -> Result<(), ProgramError> {
    if info.key.ne(&key) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !info.executable {
        return Err(ProgramError::InvalidAccountData);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use solana_program::{
        account_info::AccountInfo, keccak::Hash as KeccakHash, program_option::COption,
        program_pack::Pack, pubkey::Pubkey, system_program,
    };
    use spl_token::state::{AccountState, Mint};

    use crate::{
        loaders::{
            load_account, load_any_bus, load_bus, load_mint, load_proof, load_signer, load_sysvar,
            load_token_account, load_treasury, load_uninitialized_account, load_uninitialized_pda,
        },
        state::{Bus, Proof, Treasury},
        utils::Discriminator,
        BUS, BUS_ADDRESSES, BUS_COUNT, MINT_ADDRESS, PROOF, TOKEN_DECIMALS, TREASURY,
        TREASURY_ADDRESS,
    };

    use super::load_program;

    #[test]
    pub fn test_signer_not_signer() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_signer(&info).is_err());
    }

    #[test]
    pub fn test_load_bus_bad_account_owner() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_bus(&info, 0, true).is_err());
    }

    #[test]
    pub fn test_load_bus_bad_key() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_bus(&info, 0, true).is_err());
    }

    #[test]
    pub fn test_load_bus_empty_data() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_bus(&info, 0, true).is_err());
    }

    #[test]
    pub fn test_load_bus_bad_data() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Treasury::discriminator() as u64).to_le_bytes(), // Bus discriminator
            Bus { id: 0, rewards: 0 }.to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_bus(&info, 0, true).is_err());
    }

    #[test]
    pub fn test_load_bus_bad_id() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Bus::discriminator() as u64).to_le_bytes(), // Bus discriminator
            Bus { id: 1, rewards: 0 }.to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_bus(&info, 0, true).is_err());
    }

    #[test]
    pub fn test_load_bus_not_writeable() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Bus::discriminator() as u64).to_le_bytes(),
            Bus { id: 0, rewards: 0 }.to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_bus(&info, 0, true).is_err());
    }

    #[test]
    pub fn test_load_any_bus_bad_account_owner() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_any_bus(&info, true).is_err());
    }

    #[test]
    pub fn test_load_any_bus_bad_key() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_any_bus(&info, true).is_err());
    }

    #[test]
    pub fn test_load_any_bus_empty_data() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_any_bus(&info, true).is_err());
    }

    #[test]
    pub fn test_load_any_bus_bad_data() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Treasury::discriminator() as u64).to_le_bytes(), // Treasury discriminator
            Bus { id: 0, rewards: 0 }.to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_any_bus(&info, true).is_err());
    }

    #[test]
    pub fn test_load_any_bus_bad_id() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Bus::discriminator() as u64).to_le_bytes(),
            Bus {
                id: (BUS_COUNT + 1) as u64,
                rewards: 0,
            }
            .to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_any_bus(&info, true).is_err());
    }

    #[test]
    pub fn test_load_any_bus_mismatch_id() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Bus::discriminator() as u64).to_le_bytes(),
            Bus {
                id: 1 as u64,
                rewards: 0,
            }
            .to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_any_bus(&info, true).is_err());
    }

    #[test]
    pub fn test_load_any_bus_not_writeable() {
        let key = BUS_ADDRESSES[0];
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Bus::discriminator() as u64).to_le_bytes(),
            Bus { id: 0, rewards: 0 }.to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_any_bus(&info, true).is_err());
    }

    #[test]
    pub fn test_load_proof_bad_account_owner() {
        let authority = Pubkey::new_unique();
        let pda = Pubkey::find_program_address(&[PROOF, authority.as_ref()], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_proof(&info, &authority, true).is_err());
    }

    #[test]
    pub fn test_load_proof_bad_key() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_proof(&info, &Pubkey::new_unique(), true).is_err());
    }

    #[test]
    pub fn test_load_proof_empty_data() {
        let authority = Pubkey::new_unique();
        let pda = Pubkey::find_program_address(&[PROOF, authority.as_ref()], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_proof(&info, &authority, true).is_err());
    }

    #[test]
    pub fn test_load_proof_bad_data() {
        let authority = Pubkey::new_unique();
        let pda = Pubkey::find_program_address(&[PROOF, authority.as_ref()], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Bus::discriminator() as u64).to_le_bytes(), // Bus discriminator
            Proof {
                authority,
                claimable_rewards: 0,
                hash: KeccakHash::new_from_array([u8::MAX; 32]).into(),
                total_hashes: 0,
                total_rewards: 0,
            }
            .to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_proof(&info, &authority, true).is_err());
    }

    #[test]
    pub fn test_load_proof_not_writeable() {
        let authority = Pubkey::new_unique();
        let pda = Pubkey::find_program_address(&[PROOF, authority.as_ref()], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Proof::discriminator() as u64).to_le_bytes(),
            Proof {
                authority,
                claimable_rewards: 0,
                hash: KeccakHash::new_from_array([u8::MAX; 32]).into(),
                total_hashes: 0,
                total_rewards: 0,
            }
            .to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_proof(&info, &authority, true).is_err());
    }

    #[test]
    pub fn test_load_treasury_bad_account_owner() {
        let pda = Pubkey::find_program_address(&[TREASURY], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_treasury(&info, true).is_err());
    }

    #[test]
    pub fn test_load_treasury_bad_key() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_treasury(&info, true).is_err());
    }

    #[test]
    pub fn test_load_treasury_empty_data() {
        let key = TREASURY_ADDRESS;
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = crate::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_treasury(&info, true).is_err());
    }

    #[test]
    pub fn test_load_treasury_bad_data() {
        let pda = Pubkey::find_program_address(&[TREASURY], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Bus::discriminator() as u64).to_le_bytes(), // Bus discriminator
            Treasury {
                bump: pda.1 as u64,
                admin: Pubkey::new_unique(),
                difficulty: KeccakHash::new_from_array([u8::MAX; 32]).into(),
                last_reset_at: 0,
                reward_rate: 100,
                total_claimed_rewards: 0,
            }
            .to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_treasury(&info, true).is_err());
    }

    #[test]
    pub fn test_load_treasury_not_writeable() {
        let pda = Pubkey::find_program_address(&[TREASURY], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [
            &(Treasury::discriminator() as u64).to_le_bytes(),
            Treasury {
                bump: pda.1 as u64,
                admin: Pubkey::new_unique(),
                difficulty: KeccakHash::new_from_array([u8::MAX; 32]).into(),
                last_reset_at: 0,
                reward_rate: 100,
                total_claimed_rewards: 0,
            }
            .to_bytes(),
        ]
        .concat();
        let owner = crate::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_treasury(&info, true).is_err());
    }

    #[test]
    pub fn test_load_mint_bad_account_owner() {
        let key = MINT_ADDRESS;
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_token_account(&info, None, &MINT_ADDRESS, true).is_err());
    }

    #[test]
    pub fn test_load_mint_bad_key() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_mint(&info, true).is_err());
    }

    #[test]
    pub fn test_load_mint_empty_data() {
        let key = MINT_ADDRESS;
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_mint(&info, true).is_err());
    }

    #[test]
    pub fn test_load_mint_bad_data() {
        let key = MINT_ADDRESS;
        let mut lamports = 1_000_000_000;
        let mut data = [1];
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_mint(&info, true).is_err());
    }

    #[test]
    pub fn test_load_mint_not_writeable() {
        let mut data: [u8; Mint::LEN] = [0; Mint::LEN];
        Mint {
            mint_authority: COption::Some(TREASURY_ADDRESS),
            supply: 0,
            decimals: TOKEN_DECIMALS,
            is_initialized: true,
            freeze_authority: COption::None,
        }
        .pack_into_slice(&mut data);
        let key = MINT_ADDRESS;
        let mut lamports = 1_000_000_000;
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_mint(&info, true).is_err());
    }

    #[test]
    pub fn test_load_token_account_bad_account_owner() {
        let mut data: [u8; spl_token::state::Account::LEN] = [0; spl_token::state::Account::LEN];
        spl_token::state::Account {
            mint: MINT_ADDRESS,
            owner: TREASURY_ADDRESS,
            amount: 2_000_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        }
        .pack_into_slice(&mut data);
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_token_account(&info, None, &MINT_ADDRESS, true).is_err());
    }

    #[test]
    pub fn test_load_token_account_empty_data() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_token_account(&info, None, &MINT_ADDRESS, true).is_err());
    }

    #[test]
    pub fn test_load_token_account_bad_data() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [1];
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_token_account(&info, None, &MINT_ADDRESS, true).is_err());
    }

    #[test]
    pub fn test_load_token_account_bad_owner_mint() {
        let mut data: [u8; spl_token::state::Account::LEN] = [0; spl_token::state::Account::LEN];
        spl_token::state::Account {
            mint: MINT_ADDRESS,
            owner: TREASURY_ADDRESS,
            amount: 2_000_000_000,
            delegate: COption::None,
            state: AccountState::Initialized,
            is_native: COption::None,
            delegated_amount: 0,
            close_authority: COption::None,
        }
        .pack_into_slice(&mut data);
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_token_account(&info, Some(&key), &MINT_ADDRESS, false).is_err());
        assert!(load_token_account(&info, None, &Pubkey::new_unique(), false).is_err());
        assert!(load_token_account(&info, None, &MINT_ADDRESS, true).is_err());
    }

    #[test]
    pub fn test_load_uninitialized_pda_bad_key_bump() {
        let pda = Pubkey::find_program_address(&[TREASURY], &crate::id());
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &pda.0,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_uninitialized_pda(&info, &[BUS], pda.1, &crate::id()).is_err());
        assert!(load_uninitialized_pda(&info, &[TREASURY], 0, &crate::id()).is_err());
    }

    #[test]
    pub fn test_load_uninitialized_account_bad_owner() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = spl_token::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_uninitialized_account(&info).is_err());
    }

    #[test]
    pub fn test_load_uninitialized_account_data_not_empty() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [0];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_uninitialized_account(&info).is_err());
    }

    #[test]
    pub fn test_load_uninitialized_account_not_writeable() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_uninitialized_account(&info).is_err());
    }

    #[test]
    pub fn test_load_sysvar_bad_owner() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_sysvar(&info, key).is_err());
    }

    #[test]
    pub fn test_load_account_bad_key() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_account(&info, Pubkey::new_unique(), false).is_err());
    }

    #[test]
    pub fn test_load_account_not_writeable() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_account(&info, key, true).is_err());
    }

    #[test]
    pub fn test_load_program_bad_key() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            true,
            0,
        );
        assert!(load_program(&info, Pubkey::new_unique()).is_err());
    }

    #[test]
    pub fn test_load_program_not_executable() {
        let key = Pubkey::new_unique();
        let mut lamports = 1_000_000_000;
        let mut data = [];
        let owner = system_program::id();
        let info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        assert!(load_program(&info, key).is_err());
    }
}
