use ore_api::prelude::*;
use ore_boost_api::{consts::DENOMINATOR_BPS, prelude::Config as BoostConfig};
use steel::*;

/// Reset tops up the bus balances and updates the emissions and reward rates.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let clock = Clock::get()?;
    let (required_accounts, boost_accounts) = accounts.split_at(6);
    let [signer_info, block_info, mint_info, treasury_info, treasury_tokens_info, token_program] =
        required_accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let block = block_info
        .as_account_mut::<Block>(&ore_api::ID)?
        .assert_mut(|b| b.ends_at <= clock.slot)?
        .assert_mut(|b| b.payed_out != 0)?;
    let mint = mint_info
        .has_address(&MINT_ADDRESS)?
        .is_writable()?
        .as_mint()?;
    treasury_info.has_address(&TREASURY_ADDRESS)?;
    treasury_tokens_info.has_address(&TREASURY_TOKENS_ADDRESS)?;
    token_program.is_program(&spl_token::ID)?;

    // Load boost accounts.
    let [boost_config_info, boost_proof_info] = boost_accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let boost_config = boost_config_info.as_account::<BoostConfig>(&ore_boost_api::ID)?;
    let boost_proof = boost_proof_info
        .as_account_mut::<Proof>(&ore_api::ID)?
        .assert_mut(|p| p.authority == *boost_config_info.key)?;

    // Payout to boosts.
    let net_emissions = get_target_emissions_rate(mint.supply());
    let boost_reward =
        (net_emissions as u128 * boost_config.take_rate as u128 / DENOMINATOR_BPS as u128) as u64;
    boost_proof.balance += boost_reward;
    boost_proof.total_rewards += boost_reward;

    // Reset the block.
    block.reward = net_emissions - boost_reward;
    block.started_at = clock.slot;
    block.ends_at = clock.slot + 150; // 60 seconds
    block.payed_out = 0;
    block.total_bets = 0;
    block.bet_count = 0;
    block.noise = [0; 32];
    block.current_round += 1;

    // Fund the treasury.
    mint_to_signed(
        mint_info,
        treasury_tokens_info,
        treasury_info,
        token_program,
        net_emissions,
        &[TREASURY],
    )?;

    Ok(())
}

/// This function calculates the target emissions rate (ORE / min) based on the current supply.
/// It is designed to reduce emissions by 10% approximately every 12 months with a hardcap at 5 million ORE.
pub(crate) fn get_target_emissions_rate(current_supply: u64) -> u64 {
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
        n if n < ONE_ORE * 5_000_000 => 5_233_476_327, // Year ~29
        _ => 0,
    }
}
