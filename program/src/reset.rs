use ore_api::prelude::*;
use ore_boost_api::state::Config as BoostConfig;
use solana_program::{hash::hashv, slot_hashes::SlotHash};
use steel::*;

/// Reset tops up the bus balances and updates the emissions and reward rates.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let (required_accounts, boost_accounts) = accounts.split_at(7);
    let [signer_info, config_info, mint_info, proof_info, treasury_info, treasury_tokens_info, token_program, slot_hashes_sysvar] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let config = config_info
        .is_config()?
        .as_account_mut::<Config>(&ore_api::ID)?;
    let mint = mint_info
        .has_address(&MINT_ADDRESS)?
        .is_writable()?
        .as_mint()?;
    let proof = proof_info
        .as_account_mut::<Proof>(&ore_api::ID)?
        .assert_mut(|p| p.authority == config.best_proof)?;
    treasury_info.is_treasury()?.is_writable()?;
    treasury_tokens_info.is_treasury_tokens()?.is_writable()?;
    token_program.is_program(&spl_token::ID)?;
    slot_hashes_sysvar.is_sysvar(&sysvar::slot_hashes::ID)?;

    // Parse boost accounts.
    let [boost_config_info, boost_proof_info] = boost_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let boost_config = boost_config_info.as_account::<BoostConfig>(&ore_api::ID)?;
    let boost_proof = boost_proof_info
        .as_account_mut::<Proof>(&ore_api::ID)?
        .assert_mut(|p| p.authority == *boost_config_info.key)?;

    // Validate enough time has passed since the last reset.
    let clock = Clock::get()?;
    if config
        .last_reset_at
        .saturating_add(EPOCH_DURATION)
        .gt(&clock.unix_timestamp)
    {
        return Ok(());
    }

    // Process epoch.
    config.block_reward = get_block_reward(mint.supply());
    config.best_proof = Pubkey::default();
    config.best_difficulty = 0;
    config.last_reset_at = clock.unix_timestamp;
    config.challenge = hashv(&[
        config.challenge.as_slice(),
        &slot_hashes_sysvar.data.borrow()[0..size_of::<SlotHash>()],
    ])
    .to_bytes();

    // Calculate boost reward.
    let take_rate = boost_config.take_rate.min(9900); // Cap at 99%
    let boost_reward = config.block_reward * take_rate / ore_boost_api::consts::DENOMINATOR_BPS;
    let miner_reward = config.block_reward - boost_reward;

    // Update proof balance.
    proof.balance += miner_reward;
    boost_proof.balance += boost_reward;

    // Fund the treasury token account.
    mint_to_signed(
        mint_info,
        treasury_tokens_info,
        treasury_info,
        token_program,
        config.block_reward,
        &[TREASURY],
    )?;

    Ok(())
}

/// This function calculates the block reward (ORE / min) based on the current supply.
/// It is designed to reduce emissions by 10% approximately every 12 months with a hard stop at 5 million ORE.
pub(crate) fn get_block_reward(current_supply: u64) -> u64 {
    match current_supply {
        n if n < ONE_ORE * 525_600 => 100_000_000_000, // Year ~1
        n if n < ONE_ORE * 998_640 => 90_000_000_000,  // Year ~2
        n if n < ONE_ORE * 1_424_376 => 81_000_000_000, // Year ~3
        n if n < ONE_ORE * 1_807_538 => 72_900_000_000, // Year ~4
        n if n < ONE_ORE * 2_152_384 => 65_610_000_000, // Year ~5
        n if n < ONE_ORE * 2_462_746 => 59_049_000_000, // Year ~6
        n if n < ONE_ORE * 2_742_071 => 53_144_100_000, // Year ~7
        n if n < ONE_ORE * 2_993_464 => 47_829_690_000, // Year ~8
        n if n < ONE_ORE * 3_219_717 => 43_046_721_000, // Year ~9
        n if n < ONE_ORE * 3_423_346 => 38_742_048_900, // Year ~10
        n if n < ONE_ORE * 3_606_611 => 34_867_844_010, // Year ~11
        n if n < ONE_ORE * 3_771_550 => 31_381_059_609, // Year ~12
        n if n < ONE_ORE * 3_919_995 => 28_242_953_648, // Year ~13
        n if n < ONE_ORE * 4_053_595 => 25_418_658_283, // Year ~14
        n if n < ONE_ORE * 4_173_836 => 22_876_792_454, // Year ~15
        n if n < ONE_ORE * 4_282_052 => 20_589_113_208, // Year ~16
        n if n < ONE_ORE * 4_379_447 => 18_530_201_887, // Year ~17
        n if n < ONE_ORE * 4_467_102 => 16_677_181_698, // Year ~18
        n if n < ONE_ORE * 4_545_992 => 15_009_463_528, // Year ~19
        n if n < ONE_ORE * 4_616_993 => 13_508_517_175, // Year ~20
        n if n < ONE_ORE * 4_680_893 => 12_157_665_457, // Year ~21
        n if n < ONE_ORE * 4_738_404 => 10_941_898_911, // Year ~22
        n if n < ONE_ORE * 4_790_164 => 9_847_709_019, // Year ~23
        n if n < ONE_ORE * 4_836_747 => 8_862_938_117, // Year ~24
        n if n < ONE_ORE * 4_878_672 => 7_976_644_305, // Year ~25
        n if n < ONE_ORE * 4_916_405 => 7_178_979_874, // Year ~26
        n if n < ONE_ORE * 4_950_365 => 6_461_081_886, // Year ~27
        n if n < ONE_ORE * 4_980_928 => 5_814_973_607, // Year ~28
        n if n < MAX_SUPPLY => 5_233_476_327.min(MAX_SUPPLY - current_supply), // Year ~29
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_reward_max_supply() {
        let max_supply = ONE_ORE * 5_000_000;

        // Test at max supply
        assert_eq!(get_block_reward(max_supply), 0);

        // Test slightly below max supply
        let near_max = max_supply - 1;
        assert_eq!(get_block_reward(near_max), 1);

        // Test at max supply - 1000
        let below_max = max_supply - 1000;
        assert_eq!(get_block_reward(below_max), 1000);

        // Test that reward never exceeds remaining supply
        let supply_4_999_990 = ONE_ORE * 4_999_990;
        assert!(get_block_reward(supply_4_999_990) <= max_supply - supply_4_999_990);
    }

    #[test]
    fn test_block_reward_boundaries() {
        // Test first tier boundary
        let year1_supply = ONE_ORE * 525_599;
        assert_eq!(get_block_reward(year1_supply), 100_000_000_000);

        // Test middle tier boundary
        let year15_supply = ONE_ORE * 4_173_835;
        assert_eq!(get_block_reward(year15_supply), 22_876_792_454);

        // Test last tier boundary before max supply logic
        let last_tier_supply = ONE_ORE * 4_980_927;
        assert_eq!(get_block_reward(last_tier_supply), 5_814_973_607);
    }

    #[test]
    fn test_block_reward_zero_supply() {
        assert_eq!(get_block_reward(0), 100_000_000_000);
    }
}
