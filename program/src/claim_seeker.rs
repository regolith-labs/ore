use ore_api::{
    consts::{MINER, SEEKER, STAKE},
    state::{Miner, Seeker, Stake},
};
use solana_program::pubkey;
use steel::*;

use spl_token_2022::{
    extension::{
        metadata_pointer::MetadataPointer, BaseStateWithExtensions, PodStateWithExtensions,
    },
    pod::{PodCOption, PodMint},
};

/// Claims a Seeker genesis token for a miner.
pub fn process_claim_seeker(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, mint_info, seeker_info, stake_info, token_account_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    mint_info.has_owner(&spl_token_2022::ID)?;
    seeker_info
        .is_writable()?
        .has_seeds(&[SEEKER, &mint_info.key.to_bytes()], &ore_api::ID)?;
    stake_info.is_writable()?;
    token_account_info
        .as_associated_token_account(signer_info.key, mint_info.key)?
        .assert(|t| t.amount() == 1)?;
    system_program.is_program(&system_program::ID)?;

    // Load mint.
    let mint_data = mint_info.try_borrow_data()?;
    let mint = PodStateWithExtensions::<PodMint>::unpack(&mint_data)?;

    // Check mint authority.
    assert!(
        mint.base.mint_authority
            == PodCOption::some(pubkey!("GT2zuHVaZQYZSyQMgJPLzvkmyztfyXg2NJunqFp4p3A4")),
        "mint authority mismatch"
    );

    // Check metadata pointer.
    let ext = mint.get_extension::<MetadataPointer>()?;
    assert!(
        ext.authority.0 == pubkey!("GT2zuHVaZQYZSyQMgJPLzvkmyztfyXg2NJunqFp4p3A4"),
        "metadata authority mismatch"
    );
    assert!(
        ext.metadata_address.0 == pubkey!("GT22s89nU4iWFkNXj1Bw6uYhJJWDRPpShHt4Bk8f99Te"),
        "metadata address mismatch"
    );

    // Open seeker account.
    // Each genesis token can only be claimed once.
    if !seeker_info.data_is_empty() {
        return Ok(());
    }
    create_program_account::<Seeker>(
        seeker_info,
        system_program,
        signer_info,
        &ore_api::ID,
        &[SEEKER, &mint_info.key.to_bytes()],
    )?;
    let seeker = seeker_info.as_account_mut::<Seeker>(&ore_api::ID)?;
    seeker.mint = *mint_info.key;

    // Open stake account.
    let stake = if stake_info.data_is_empty() {
        create_program_account::<Stake>(
            stake_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[STAKE, &signer_info.key.to_bytes()],
        )?;
        let stake = stake_info.as_account_mut::<Stake>(&ore_api::ID)?;
        stake.authority = *signer_info.key;
        stake.balance = 0;
        stake.last_claim_at = 0;
        stake.last_deposit_at = 0;
        stake.last_withdraw_at = 0;
        stake.rewards_factor = Numeric::from_u64(0);
        stake.rewards = 0;
        stake.lifetime_rewards = 0;
        stake.is_seeker = 0;
        stake
    } else {
        stake_info
            .as_account_mut::<Stake>(&ore_api::ID)?
            .assert_mut(|s| s.authority == *signer_info.key)?
            .assert_mut(|s| s.is_seeker == 0)?
    };

    // Flag the miner as a Seeker.
    stake.is_seeker = 1;

    Ok(())
}
