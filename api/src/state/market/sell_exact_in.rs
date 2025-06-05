use crate::error::OreError;

use super::{Market, SwapDirection, TokenType, VirtualLimitOrder};
use crate::event::SwapEvent;

impl Market {
    pub fn sell_exact_in(&mut self, base_in: u64) -> Result<SwapEvent, OreError> {
        // Get fee from quote side.
        let mut quote_fee = 0;

        // Upcast data.
        let base_in = base_in as u128;

        // Get virtual limit order.
        let VirtualLimitOrder {
            size_in_base: bid_size_in_base,
            size_in_quote: bid_size_in_quote,
        } = self.get_virtual_limit_order(SwapDirection::Sell);

        // Execute swap.
        let (base_via_bid, quote_via_bid, base_via_curve, quote_via_curve) =
            if !self.sandwich_resistance_enabled() {
                // Fill entire swap via curve.
                let base_via_curve = base_in;
                let mut quote_via_curve = self.get_quote_out(base_via_curve);
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Sell);
                let swap_fee = self.fee(quote_via_curve as u64);
                quote_fee += swap_fee;
                quote_via_curve -= swap_fee as u128;
                (0, 0, base_via_curve, quote_via_curve)
            } else if bid_size_in_base >= base_in {
                // Fill entire swap through virtual limit order.
                let base_via_bid = base_in;
                let mut quote_via_bid = self.get_complementary_limit_order_size(
                    base_in,
                    SwapDirection::Sell,
                    TokenType::Base,
                );
                quote_fee += self.fee(quote_via_bid as u64);
                self.update_reserves(base_via_bid, quote_via_bid, SwapDirection::Sell);
                quote_via_bid -= quote_fee as u128;
                (base_via_bid, quote_via_bid, 0, 0)
            } else {
                // Partially fill swap through virtual limit order.
                let base_via_bid = bid_size_in_base;
                let mut quote_via_bid = bid_size_in_quote;
                quote_fee += self.fee(quote_via_bid as u64);
                self.update_reserves(base_via_bid, quote_via_bid, SwapDirection::Sell);
                quote_via_bid -= quote_fee as u128;

                // Fill remaining swap through pool.
                let base_via_curve = base_in - base_via_bid;
                let mut quote_via_curve = self.get_quote_out(base_via_curve);
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Sell);
                let swap_fee = self.fee(quote_via_curve as u64);
                quote_fee += swap_fee;
                quote_via_curve -= swap_fee as u128;
                (base_via_bid, quote_via_bid, base_via_curve, quote_via_curve)
            };

        // Produce swap result.
        let quote_out = quote_via_bid + quote_via_curve;
        let swap_event = SwapEvent {
            direction: SwapDirection::Sell as u64,
            base_to_transfer: base_in as u64,
            quote_to_transfer: quote_out as u64,
            base_via_order: base_via_bid as u64,
            quote_via_order: quote_via_bid as u64,
            base_via_curve: base_via_curve as u64,
            quote_via_curve: quote_via_curve as u64,
            quote_fee: quote_fee as u64,
        };

        // Sanity check swap result.
        assert!(
            swap_event.base_to_transfer == swap_event.base_via_order + swap_event.base_via_curve
        );
        assert!(
            swap_event.quote_to_transfer == swap_event.quote_via_order + swap_event.quote_via_curve
        );

        // Return.
        Ok(swap_event)
    }
}
