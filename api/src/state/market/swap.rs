use steel::Clock;

use crate::error::OreError;

use super::{Market, SwapDirection, SwapPrecision};
use crate::event::SwapEvent;

impl Market {
    pub fn swap(
        &mut self,
        amount: u64,
        direction: SwapDirection,
        precision: SwapPrecision,
        clock: Clock,
    ) -> Result<SwapEvent, OreError> {
        // Update snapshot.
        self.update_snapshot(clock);

        // Get invariant.
        let k_pre = self.k();

        // Execute swap.
        let swap_event = match (direction, precision) {
            (SwapDirection::Buy, SwapPrecision::ExactIn) => self.buy_exact_in(amount)?,
            (SwapDirection::Buy, SwapPrecision::ExactOut) => self.buy_exact_out(amount)?,
            (SwapDirection::Sell, SwapPrecision::ExactIn) => self.sell_exact_in(amount)?,
            (SwapDirection::Sell, SwapPrecision::ExactOut) => self.sell_exact_out(amount)?,
        };

        // Check invariant.
        let k_post = self.k();
        if k_pre > k_post {
            return Err(OreError::InvariantViolation.into());
        }

        // Apply fees.
        self.apply_fees(swap_event.quote_fee);

        // Return.
        Ok(swap_event)
    }
}
