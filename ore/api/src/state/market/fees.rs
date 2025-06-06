use crate::consts::*;

use super::Market;

impl Market {
    pub(crate) fn apply_fees(&mut self, quote_fee: u64) {
        // Process protocol fees.
        self.fee.cumulative += quote_fee;
        self.fee.uncollected += quote_fee;
    }

    /// Calculates the fee from a quote amount.
    pub(crate) fn fee(&self, quote_size: u64) -> u64 {
        quote_size * self.fee.rate / DENOMINATOR_BPS
    }

    /// Calculates the pre-fee quote from a post-fee quote amount.
    pub(crate) fn pre_fee(&self, quote_post_fee: u64) -> u64 {
        // x * 10000 / (10000 - fee) is approximately equivalent to x * (1 - fee / 10000)
        let numerator = quote_post_fee * DENOMINATOR_BPS;
        let denominator = DENOMINATOR_BPS - self.fee.rate;
        return numerator / denominator;
    }
}
