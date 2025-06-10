use crate::error::OreError;

use super::{Market, SwapDirection, TokenType, VirtualLimitOrder};
use crate::event::SwapEvent;

impl Market {
    pub fn buy_exact_in(&mut self, quote_in: u64) -> Result<SwapEvent, OreError> {
        // Get fee from quote side.
        let quote_fee = self.fee(quote_in);
        let quote_in_post_fee = quote_in - quote_fee;

        // Upcast data.
        let quote_in_post_fee = quote_in_post_fee as u128;

        // Get virtual limit order.
        let VirtualLimitOrder {
            size_in_base: ask_size_in_base,
            size_in_quote: ask_size_in_quote,
        } = self.get_virtual_limit_order(SwapDirection::Buy);

        // Execute swap.
        let (base_via_ask, quote_via_ask, base_via_curve, quote_via_curve) =
            if !self.sandwich_resistance_enabled() {
                // Fill entire swap via curve.
                let quote_via_curve = quote_in_post_fee;
                let base_via_curve = self.get_base_out(quote_via_curve);
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Buy)?;
                (0, 0, base_via_curve, quote_via_curve)
            } else if ask_size_in_quote >= quote_in_post_fee {
                // Fill entire swap via virtual limit order.
                let quote_via_ask = quote_in_post_fee;
                let base_via_ask = self.get_complementary_limit_order_size(
                    quote_in_post_fee,
                    SwapDirection::Buy,
                    TokenType::Quote,
                );
                self.update_reserves(base_via_ask, quote_via_ask, SwapDirection::Buy)?;
                (base_via_ask, quote_via_ask, 0, 0)
            } else {
                // Partially fill swap via virtual limit order.
                let base_via_ask = ask_size_in_base;
                let quote_via_ask = ask_size_in_quote;
                self.update_reserves(base_via_ask, quote_via_ask, SwapDirection::Buy)?;

                // Fill remaining swap amount via curve.
                let quote_via_curve = quote_in_post_fee - ask_size_in_quote;
                let base_via_curve = self.get_base_out(quote_via_curve);
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Buy)?;
                (base_via_ask, quote_via_ask, base_via_curve, quote_via_curve)
            };

        // Produce swap result.
        let base_out = base_via_ask + base_via_curve;
        let swap_event = SwapEvent {
            block_id: 0,
            direction: SwapDirection::Buy as u64,
            base_to_transfer: base_out as u64,
            quote_to_transfer: quote_in,
            base_via_order: base_via_ask as u64,
            quote_via_order: quote_via_ask as u64,
            base_via_curve: base_via_curve as u64,
            quote_via_curve: quote_via_curve as u64,
            quote_fee: quote_fee as u64,
        };

        // Sanity check swap event.
        assert!(
            swap_event.base_to_transfer == swap_event.base_via_order + swap_event.base_via_curve
        );
        assert!(
            swap_event.quote_to_transfer
                == swap_event.quote_via_order + swap_event.quote_via_curve + swap_event.quote_fee
        );

        // Return
        Ok(swap_event)
    }
}
