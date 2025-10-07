use ore_api::prelude::*;
use steel::*;

pub fn process_migrate_miner(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, config_info, miner_info, stake_info, treasury_info, system_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .as_account_mut::<Config>(&ore_api::ID)?
        .assert_mut_err(
            |c| c.admin == *signer_info.key,
            OreError::NotAuthorized.into(),
        )?;
    let treasury = treasury_info.as_account_mut::<Treasury>(&ore_api::ID)?;
    system_program.is_program(&system_program::ID)?;

    let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;

    let stake = if stake_info.data_is_empty() {
        create_program_account::<Stake>(
            stake_info,
            system_program,
            &signer_info,
            &ore_api::ID,
            &[STAKE, &miner.authority.to_bytes()],
        )?;
        let stake = stake_info.as_account_mut::<Stake>(&ore_api::ID)?;
        stake.authority = miner.authority;
        stake.balance = 0;
        stake.last_claim_at = 0;
        stake.last_deposit_at = 0;
        stake.last_withdraw_at = 0;
        stake.rewards_factor = treasury.rewards_factor;
        stake.rewards = 0;
        stake.lifetime_rewards = 0;
        stake.is_seeker = 0;
        stake
    } else {
        stake_info
            .as_account_mut::<Stake>(&ore_api::ID)?
            .assert_mut(|s| s.authority == miner.authority)?
    };

    // Update values.
    treasury.total_unclaimed += miner.rewards_ore;

    Ok(())
}
