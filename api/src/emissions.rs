use ore_mint_api::consts::MAX_SUPPLY;
use steel::Numeric;

use crate::consts::ONE_ORE;

/// Base gross emission rate at zero circulating supply, in token grams per round.
/// Equal to 1.2 ORE.
pub const BASE_EMISSION_RATE: u64 = ONE_ORE * 6 / 5;

/// Returns `(main, motherlode, total)` mint amounts for a round given current circulating supply.
///
/// Total rate per round:
/// `1.2 * sqrt(remaining / MAX_SUPPLY)` ORE, where `remaining = MAX_SUPPLY - circulating`.
///
/// The total is split 5:1 (main : motherlode). Amounts are floored to whole grams and
/// clamped so `total <= remaining`.
pub fn emission_amounts(circulating: u64) -> (u64, u64, u64) {
    let remaining = MAX_SUPPLY.saturating_sub(circulating);
    if remaining == 0 {
        return (0, 0, 0);
    }

    // total = 1.2 ORE * sqrt(remaining / MAX_SUPPLY)
    let total = Numeric::from_u64(BASE_EMISSION_RATE)
        * Numeric::from_fraction(remaining, MAX_SUPPLY).sqrt();
    let total = total.to_u64().min(remaining);

    let main = total * 5 / 6;
    let motherlode = total - main;
    (main, motherlode, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use spl_token::amount_to_ui_amount;

    /// Tolerance of 0.0001 ORE in grams for fixed-point rounding vs the spec table.
    const TOLERANCE: u64 = ONE_ORE / 10_000;

    fn ore(amount: f64) -> u64 {
        (amount * ONE_ORE as f64).round() as u64
    }

    #[test]
    fn emission_at_zero_supply_is_base_rate() {
        let (main, motherlode, total) = emission_amounts(0);
        assert_eq!(total, BASE_EMISSION_RATE);
        assert_eq!(main, ONE_ORE);
        assert_eq!(motherlode, ONE_ORE / 5);
        assert_eq!(main + motherlode, total);
    }

    #[test]
    fn emission_at_max_supply_is_zero() {
        assert_eq!(emission_amounts(MAX_SUPPLY), (0, 0, 0));
        assert_eq!(emission_amounts(MAX_SUPPLY + 1), (0, 0, 0));
    }

    #[test]
    fn emission_never_exceeds_remaining() {
        let remaining = 1_000;
        let circulating = MAX_SUPPLY - remaining;
        let (_main, _motherlode, total) = emission_amounts(circulating);
        assert!(total <= remaining);
    }

    #[test]
    fn emission_matches_spec_table() {
        // Circulating (ORE millions), expected total rate (ORE / round).
        let cases = [
            (0.00, 1.2000),
            (0.50, 1.0954),
            (1.00, 0.9798),
            (1.50, 0.8485),
            (2.00, 0.6928),
            (2.50, 0.4899),
            (2.80, 0.3098),
            (2.90, 0.2191),
            (2.95, 0.1549),
        ];

        for (circulating_m, expected_total) in cases {
            let circulating = ore(circulating_m * 1_000_000.0);
            let (main, motherlode, total) = emission_amounts(circulating);
            let expected = ore(expected_total);

            assert!(
                total.abs_diff(expected) <= TOLERANCE,
                "circulating={circulating_m}M: total {} ORE, expected {expected_total} ORE",
                amount_to_ui_amount(total, 11),
            );
            assert_eq!(main + motherlode, total);
            assert_eq!(main, total * 5 / 6);
            assert_eq!(motherlode, total - main);
        }
    }
}
