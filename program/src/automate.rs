use ore_api::prelude::*;
use steel::*;

/// Sets the executor.
pub fn process_automate(accounts: &[AccountInfo<'_>], data: &[u8]) -> ProgramResult {
    // Parse data.
    let args = Automate::try_from_bytes(data)?;
    let amount = u64::from_le_bytes(args.amount);
    let deposit = u64::from_le_bytes(args.deposit);
    let fee = u64::from_le_bytes(args.fee);
    let mask = u64::from_le_bytes(args.mask);
    let strategy = AutomationStrategy::from_u64(args.strategy as u64);

    // Load accounts.
    let [signer_info, automation_info, executor_info, miner_info, system_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    automation_info.is_writable()?;
    system_program.is_program(&system_program::ID)?;

    // // Check whitelist
    // if !AUTHORIZED_ACCOUNTS.contains(&signer_info.key) {
    //     return Err(trace("Not authorized", OreError::NotAuthorized.into()));
    // }

    // Open miner account.
    let miner = if miner_info.data_is_empty() {
        create_program_account::<Miner>(
            miner_info,
            system_program,
            &signer_info,
            &ore_api::ID,
            &[MINER, &signer_info.key.to_bytes()],
        )?;
        let miner = miner_info.as_account_mut::<Miner>(&ore_api::ID)?;
        miner.authority = *signer_info.key;
        miner.deployed = [0; 25];
        miner.cumulative = [0; 25];
        miner.checkpoint_fee = 0;
        miner.checkpoint_id = 0;
        miner.rewards_sol = 0;
        miner.rewards_ore = 0;
        miner.round_id = 0;
        miner.lifetime_rewards_sol = 0;
        miner.lifetime_rewards_ore = 0;
        miner
    } else {
        miner_info
            .as_account_mut::<Miner>(&ore_api::ID)?
            .assert_mut_err(
                |m| m.authority == *signer_info.key,
                OreError::NotAuthorized.into(),
            )?
    };

    // Close account if executor is Pubkey::default().
    if *executor_info.key == Pubkey::default() {
        automation_info
            .as_account_mut::<Automation>(&ore_api::ID)?
            .assert_mut_err(
                |a| a.authority == *signer_info.key,
                OreError::NotAuthorized.into(),
            )?;
        automation_info.close(signer_info)?;
        return Ok(());
    }

    // Create automation.
    let automation = if automation_info.data_is_empty() {
        create_program_account::<Automation>(
            automation_info,
            system_program,
            signer_info,
            &ore_api::ID,
            &[AUTOMATION, &signer_info.key.to_bytes()],
        )?;
        let automation = automation_info.as_account_mut::<Automation>(&ore_api::ID)?;
        automation.balance = 0;
        automation.authority = *signer_info.key;
        automation
    } else {
        automation_info
            .as_account_mut::<Automation>(&ore_api::ID)?
            .assert_mut_err(
                |a| a.authority == *signer_info.key,
                OreError::NotAuthorized.into(),
            )?
    };

    // Set strategy and mask.
    automation.amount = amount;
    automation.balance += deposit;
    automation.executor = *executor_info.key;
    automation.fee = fee;
    automation.mask = mask;
    automation.strategy = strategy as u64;

    // Top up checkpoint fee.
    if miner.checkpoint_fee == 0 {
        miner.checkpoint_fee = CHECKPOINT_FEE;
        miner_info.collect(CHECKPOINT_FEE, &signer_info)?;
    }

    // Transfer balance to executor.
    automation_info.collect(deposit, signer_info)?;

    Ok(())
}
