use ore_api::prelude::*;
use steel::*;

/// Reset tops up the bus balances and updates the emissions and reward rates.
pub fn process_reset(accounts: &[AccountInfo<'_>], _data: &[u8]) -> ProgramResult {
    // Load accounts.
    let [signer_info, bus_0_info, bus_1_info, bus_2_info, bus_3_info, bus_4_info, bus_5_info, bus_6_info, bus_7_info, config_info, mint_info, treasury_info, treasury_tokens_info, token_program] =
        accounts
    else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    signer_info.is_signer()?;
    let bus_0 = bus_0_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 0)?;
    let bus_1 = bus_1_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 1)?;
    let bus_2 = bus_2_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 2)?;
    let bus_3 = bus_3_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 3)?;
    let bus_4 = bus_4_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 4)?;
    let bus_5 = bus_5_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 5)?;
    let bus_6 = bus_6_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 6)?;
    let bus_7 = bus_7_info
        .as_account_mut::<Bus>(&ore_api::ID)?
        .assert_mut(|b| b.id == 7)?;
    let config = config_info
        .is_config()?
        .as_account_mut::<Config>(&ore_api::ID)?;
    let mint = mint_info
        .has_address(&MINT_ADDRESS)?
        .is_writable()?
        .as_mint()?;
    treasury_info.is_treasury()?.is_writable()?;
    treasury_tokens_info.is_treasury_tokens()?.is_writable()?;
    token_program.is_program(&spl_token::ID)?;

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
    let busses = [bus_0, bus_1, bus_2, bus_3, bus_4, bus_5, bus_6, bus_7];
    let amount_to_mint = config.process_epoch(busses, &clock, &mint)?;

    // Fund the treasury token account.
    mint_to_signed(
        mint_info,
        treasury_tokens_info,
        treasury_info,
        token_program,
        amount_to_mint,
        &[TREASURY],
    )?;

    Ok(())
}

trait EpochProcessor {
    fn process_epoch(
        &mut self,
        busses: [&mut Bus; 8],
        clock: &Clock,
        mint: &Mint,
    ) -> Result<u64, ProgramError>;
}

impl EpochProcessor for Config {
    fn process_epoch(
        &mut self,
        busses: [&mut Bus; 8],
        clock: &Clock,
        mint: &Mint,
    ) -> Result<u64, ProgramError> {
        // Max supply check.
        if mint.supply() >= MAX_SUPPLY {
            return Err(OreError::MaxSupply.into());
        }

        // Update timestamp.
        self.last_reset_at = clock.unix_timestamp;

        // Adjust emissions curve based on current supply.
        self.target_emmissions_rate = get_target_emissions_rate(mint.supply());

        // Calculate target rewards to distribute in coming epoch (emissions rate multiplied by epoch duration).
        let target_epoch_rewards = self.target_emmissions_rate * EPOCH_MINUTES as u64;

        // Reset bus counters and calculate theoretical rewards mined in the last epoch.
        let mut amount_to_mint = 0u64;
        let mut remaining_supply = MAX_SUPPLY.saturating_sub(mint.supply());
        let mut theoretical_epoch_rewards = 0u64;
        for bus in busses {
            // Reset theoretical rewards.
            theoretical_epoch_rewards += bus.theoretical_rewards;
            bus.theoretical_rewards = 0;

            // Reset bus rewards.
            let topup_amount = target_epoch_rewards
                .saturating_sub(bus.rewards)
                .min(remaining_supply);
            remaining_supply -= topup_amount;
            amount_to_mint += topup_amount;
            bus.rewards += topup_amount;
        }

        // Update base reward rate for next epoch.
        self.base_reward_rate = calculate_new_reward_rate(
            self.base_reward_rate,
            theoretical_epoch_rewards,
            target_epoch_rewards,
        );

        // If base reward rate is too low, increment min difficulty by 1 and double base reward rate.
        if self.base_reward_rate < BASE_REWARD_RATE_MIN_THRESHOLD {
            self.min_difficulty += 1;
            self.base_reward_rate *= 2;
        }

        // If base reward rate is too high, decrement min difficulty by 1 and halve base reward rate.
        if self.base_reward_rate >= BASE_REWARD_RATE_MAX_THRESHOLD && self.min_difficulty > 1 {
            self.min_difficulty -= 1;
            self.base_reward_rate /= 2;
        }

        Ok(amount_to_mint)
    }
}

/// This function calculates what the new reward rate should be based on how many total rewards
/// were mined in the prior epoch. The math is largely identitical to function used by the Bitcoin
/// network to update the difficulty between each epoch.
///
/// new_rate = current_rate * (target_rewards / actual_rewards)
///
/// The new rate is then smoothed by a constant factor to avoid large fluctuations. In Ore's case,
/// the epochs are short (60 seconds) so a smoothing factor of 2 has been chosen. That is, the reward rate
/// can at most double or halve from one epoch to the next.
pub(crate) fn calculate_new_reward_rate(
    current_rate: u64,
    epoch_rewards: u64,
    target_epoch_rewards: u64,
) -> u64 {
    // Avoid division by zero. Leave the reward rate unchanged, if detected.
    if epoch_rewards.eq(&0) {
        return current_rate;
    }

    // Calculate new reward rate.
    let new_rate = (current_rate as u128)
        .saturating_mul(target_epoch_rewards as u128)
        .saturating_div(epoch_rewards as u128) as u64;

    // Smooth reward rate so it cannot change by more than a constant factor from one epoch to the next.
    let new_rate_min = current_rate.saturating_div(SMOOTHING_FACTOR);
    let new_rate_max = current_rate.saturating_mul(SMOOTHING_FACTOR);
    let new_rate_smoothed = new_rate.min(new_rate_max).max(new_rate_min);

    // Prevent reward rate from dropping below 1 or exceeding BUS_EPOCH_REWARDS and return.
    new_rate_smoothed.max(1).min(target_epoch_rewards)
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

#[cfg(test)]
mod tests {
    use rand::{distributions::Uniform, Rng};
    use solana_program::program_option::COption;
    use steel::{Clock, Mint};

    use crate::{calculate_new_reward_rate, reset::EpochProcessor};
    use ore_api::{
        consts::{
            BASE_REWARD_RATE_MIN_THRESHOLD, BUS_COUNT, EPOCH_MINUTES, ONE_ORE, SMOOTHING_FACTOR,
            TOKEN_DECIMALS,
        },
        state::{Bus, Config},
    };

    const FUZZ_SIZE: u64 = 10_000;
    const TARGET_EPOCH_REWARDS: u64 = ONE_ORE * EPOCH_MINUTES as u64;
    const MAX_EPOCH_REWARDS: u64 = TARGET_EPOCH_REWARDS * BUS_COUNT as u64;

    #[test]
    fn test_calculate_new_reward_rate_target() {
        let current_rate = 1000;
        let new_rate =
            calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_div_by_zero() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 0, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_EPOCH_REWARDS.saturating_add(1_000_000_000),
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower_edge() {
        let current_rate = BASE_REWARD_RATE_MIN_THRESHOLD;
        let new_rate =
            calculate_new_reward_rate(current_rate, TARGET_EPOCH_REWARDS + 1, TARGET_EPOCH_REWARDS);
        assert!(new_rate.lt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_lower_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, TARGET_EPOCH_REWARDS));
            let actual_rewards: u64 =
                rng.sample(Uniform::new(TARGET_EPOCH_REWARDS, MAX_EPOCH_REWARDS));
            let new_rate =
                calculate_new_reward_rate(current_rate, actual_rewards, TARGET_EPOCH_REWARDS);
            assert!(new_rate.lt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_higher() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(
            current_rate,
            TARGET_EPOCH_REWARDS.saturating_sub(1_000_000_000),
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.gt(&current_rate));
    }

    #[test]
    fn test_calculate_new_reward_rate_higher_fuzz() {
        let mut rng = rand::thread_rng();
        for _ in 0..FUZZ_SIZE {
            let current_rate: u64 = rng.sample(Uniform::new(1, TARGET_EPOCH_REWARDS));
            let actual_rewards: u64 = rng.sample(Uniform::new(1, TARGET_EPOCH_REWARDS));
            let new_rate =
                calculate_new_reward_rate(current_rate, actual_rewards, TARGET_EPOCH_REWARDS);
            assert!(new_rate.gt(&current_rate));
        }
    }

    #[test]
    fn test_calculate_new_reward_rate_max_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, 1, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate.saturating_mul(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_smooth() {
        let current_rate = 1000;
        let new_rate = calculate_new_reward_rate(current_rate, u64::MAX, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&current_rate.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_max_inputs() {
        let new_rate = calculate_new_reward_rate(
            TARGET_EPOCH_REWARDS,
            MAX_EPOCH_REWARDS,
            TARGET_EPOCH_REWARDS,
        );
        assert!(new_rate.eq(&TARGET_EPOCH_REWARDS.saturating_div(SMOOTHING_FACTOR)));
    }

    #[test]
    fn test_calculate_new_reward_rate_min_inputs() {
        let new_rate = calculate_new_reward_rate(1, 1, TARGET_EPOCH_REWARDS);
        assert!(new_rate.eq(&1u64.saturating_mul(SMOOTHING_FACTOR)));
    }

    #[allow(deprecated)]
    #[test]
    fn test_process_epoch_simple() {
        let mut config = Config {
            base_reward_rate: 1024,
            last_reset_at: 0,
            min_difficulty: 1,
            target_emmissions_rate: ONE_ORE,
        };
        let bus_0 = &mut Bus {
            id: 0,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_1 = &mut Bus {
            id: 1,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_2 = &mut Bus {
            id: 2,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_3 = &mut Bus {
            id: 3,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_4 = &mut Bus {
            id: 4,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_5 = &mut Bus {
            id: 5,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_6 = &mut Bus {
            id: 6,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_7 = &mut Bus {
            id: 7,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let busses = [bus_0, bus_1, bus_2, bus_3, bus_4, bus_5, bus_6, bus_7];
        let clock = Clock::default();
        let mint = Mint::V0(spl_token::state::Mint {
            mint_authority: COption::None,
            supply: ONE_ORE * 100,
            decimals: TOKEN_DECIMALS,
            is_initialized: true,
            freeze_authority: COption::None,
        });

        let amount_to_mint = config.process_epoch(busses, &clock, &mint).unwrap();
        assert_eq!(config.target_emmissions_rate, ONE_ORE);
        assert_eq!(
            ONE_ORE * EPOCH_MINUTES as u64 * BUS_COUNT as u64,
            amount_to_mint
        );
    }

    #[allow(deprecated)]
    #[test]
    fn test_process_epoch_emissions_boundary() {
        let mut config = Config {
            base_reward_rate: 1024,
            last_reset_at: 0,
            min_difficulty: 1,
            target_emmissions_rate: ONE_ORE,
        };
        let bus_0 = &mut Bus {
            id: 0,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_1 = &mut Bus {
            id: 1,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_2 = &mut Bus {
            id: 2,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_3 = &mut Bus {
            id: 3,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_4 = &mut Bus {
            id: 4,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_5 = &mut Bus {
            id: 5,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_6 = &mut Bus {
            id: 6,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_7 = &mut Bus {
            id: 7,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let busses = [bus_0, bus_1, bus_2, bus_3, bus_4, bus_5, bus_6, bus_7];
        let clock = Clock::default();
        let mint = Mint::V0(spl_token::state::Mint {
            mint_authority: COption::None,
            supply: ONE_ORE * 525_600,
            decimals: TOKEN_DECIMALS,
            is_initialized: true,
            freeze_authority: COption::None,
        });

        let amount_to_mint = config.process_epoch(busses, &clock, &mint).unwrap();
        assert_eq!(config.target_emmissions_rate, 90_000_000_000);
        assert_eq!(
            90_000_000_000 * EPOCH_MINUTES as u64 * BUS_COUNT as u64,
            amount_to_mint
        );
    }

    #[allow(deprecated)]
    #[test]
    fn test_process_epoch_max_supply() {
        let mut config = Config {
            base_reward_rate: 1024,
            last_reset_at: 0,
            min_difficulty: 1,
            target_emmissions_rate: 5_233_476_327,
        };
        let bus_0 = &mut Bus {
            id: 0,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_1 = &mut Bus {
            id: 1,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_2 = &mut Bus {
            id: 2,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_3 = &mut Bus {
            id: 3,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_4 = &mut Bus {
            id: 4,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_5 = &mut Bus {
            id: 5,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_6 = &mut Bus {
            id: 6,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let bus_7 = &mut Bus {
            id: 7,
            rewards: 0,
            theoretical_rewards: 0,
            top_balance: 0,
        };
        let busses = [bus_0, bus_1, bus_2, bus_3, bus_4, bus_5, bus_6, bus_7];
        let clock = Clock::default();
        let mint = Mint::V0(spl_token::state::Mint {
            mint_authority: COption::None,
            supply: ONE_ORE * 4_999_999,
            decimals: TOKEN_DECIMALS,
            is_initialized: true,
            freeze_authority: COption::None,
        });

        let amount_to_mint = config.process_epoch(busses, &clock, &mint).unwrap();
        assert_eq!(config.target_emmissions_rate, 5_233_476_327);
        assert_eq!(ONE_ORE, amount_to_mint);
    }
}
