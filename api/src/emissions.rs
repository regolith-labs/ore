use ore_mint_api::consts::MAX_SUPPLY;

use crate::consts::ONE_ORE;

/// Circulating supply at which the first emission cut applies.
pub const EMISSION_CUT_START: u64 = 600_000 * ONE_ORE;

/// Circulating supply step size between emission cuts.
pub const EMISSION_CUT_STEP: u64 = 100_000 * ONE_ORE;

/// Main reward before any cuts (1.00 ORE).
pub const MAIN_BASE: u64 = ONE_ORE;

/// Motherlode reward before any cuts (0.20 ORE).
pub const MOTHERLODE_BASE: u64 = ONE_ORE / 5;

/// Main reward reduction per step (0.05 ORE).
pub const MAIN_STEP: u64 = ONE_ORE / 20;

/// Motherlode reward reduction per step (0.01 ORE).
pub const MOTHERLODE_STEP: u64 = ONE_ORE / 100;

/// Floor for main reward (0.05 ORE).
pub const MAIN_FLOOR: u64 = ONE_ORE / 20;

/// Floor for motherlode reward (0.01 ORE).
pub const MOTHERLODE_FLOOR: u64 = ONE_ORE / 100;

/// Returns `(main, motherlode, total)` mint amounts for a round given circulating supply.
///
/// Schedule (per round):
/// - Below 600k: main = 1.00, motherlode = 0.20
/// - At 600k, then every additional 100k: main −0.05, motherlode −0.01
/// - Floor at 2.4M+: main = 0.05, motherlode = 0.01
///
/// Amounts are clamped so `total <= remaining` supply under `MAX_SUPPLY`.
pub fn emission_amounts(circulating: u64) -> (u64, u64, u64) {
    let remaining = MAX_SUPPLY.saturating_sub(circulating);
    if remaining == 0 {
        return (0, 0, 0);
    }

    let (mut main, mut motherlode) = if circulating < EMISSION_CUT_START {
        (MAIN_BASE, MOTHERLODE_BASE)
    } else {
        // steps = 0 for [600k, 700k), 1 for [700k, 800k), …, 18 for [2.4M, ∞)
        let steps = (circulating - EMISSION_CUT_START) / EMISSION_CUT_STEP;
        let cuts = steps + 1;
        let main = MAIN_BASE.saturating_sub(MAIN_STEP.saturating_mul(cuts)).max(MAIN_FLOOR);
        let motherlode = MOTHERLODE_BASE
            .saturating_sub(MOTHERLODE_STEP.saturating_mul(cuts))
            .max(MOTHERLODE_FLOOR);
        (main, motherlode)
    };

    let mut total = main + motherlode;
    if total > remaining {
        // Prefer preserving main when trimming against remaining supply.
        main = main.min(remaining);
        motherlode = remaining.saturating_sub(main);
        total = remaining;
    }

    (main, motherlode, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spl_token::amount_to_ui_amount;

    fn ore(amount: f64) -> u64 {
        (amount * ONE_ORE as f64).round() as u64
    }

    #[test]
    fn below_cut_start_is_base_rate() {
        assert_eq!(emission_amounts(0), (MAIN_BASE, MOTHERLODE_BASE, MAIN_BASE + MOTHERLODE_BASE));
        assert_eq!(
            emission_amounts(EMISSION_CUT_START - 1),
            (MAIN_BASE, MOTHERLODE_BASE, MAIN_BASE + MOTHERLODE_BASE)
        );
    }

    #[test]
    fn at_max_supply_is_zero() {
        assert_eq!(emission_amounts(MAX_SUPPLY), (0, 0, 0));
        assert_eq!(emission_amounts(MAX_SUPPLY + 1), (0, 0, 0));
    }

    #[test]
    fn never_exceeds_remaining() {
        let remaining = 1_000;
        let circulating = MAX_SUPPLY - remaining;
        let (main, motherlode, total) = emission_amounts(circulating);
        assert!(total <= remaining);
        assert_eq!(main + motherlode, total);
    }

    #[test]
    fn matches_schedule_table() {
        // (circulating lower bound in ORE, main, motherlode)
        let cases = [
            (0.0, 1.00, 0.20),
            (599_999.0, 1.00, 0.20),
            (600_000.0, 0.95, 0.19),
            (700_000.0, 0.90, 0.18),
            (800_000.0, 0.85, 0.17),
            (900_000.0, 0.80, 0.16),
            (1_000_000.0, 0.75, 0.15),
            (1_100_000.0, 0.70, 0.14),
            (1_200_000.0, 0.65, 0.13),
            (1_300_000.0, 0.60, 0.12),
            (1_400_000.0, 0.55, 0.11),
            (1_500_000.0, 0.50, 0.10),
            (1_600_000.0, 0.45, 0.09),
            (1_700_000.0, 0.40, 0.08),
            (1_800_000.0, 0.35, 0.07),
            (1_900_000.0, 0.30, 0.06),
            (2_000_000.0, 0.25, 0.05),
            (2_100_000.0, 0.20, 0.04),
            (2_200_000.0, 0.15, 0.03),
            (2_300_000.0, 0.10, 0.02),
            (2_400_000.0, 0.05, 0.01),
            (2_900_000.0, 0.05, 0.01),
        ];

        for (circulating_ore, expected_main, expected_ml) in cases {
            let circulating = ore(circulating_ore);
            let (main, motherlode, total) = emission_amounts(circulating);
            assert_eq!(
                main,
                ore(expected_main),
                "circulating={circulating_ore}: main {} ORE, expected {expected_main}",
                amount_to_ui_amount(main, 11),
            );
            assert_eq!(
                motherlode,
                ore(expected_ml),
                "circulating={circulating_ore}: motherlode {} ORE, expected {expected_ml}",
                amount_to_ui_amount(motherlode, 11),
            );
            assert_eq!(main + motherlode, total);
        }
    }

    #[test]
    fn range_upper_bounds_stay_in_prior_bucket() {
        // Just below the next cut should still use the previous step.
        assert_eq!(
            emission_amounts(ore(700_000.0) - 1),
            (ore(0.95), ore(0.19), ore(1.14))
        );
        assert_eq!(
            emission_amounts(ore(2_400_000.0) - 1),
            (ore(0.10), ore(0.02), ore(0.12))
        );
    }
}
