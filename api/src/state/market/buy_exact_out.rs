use steel::Pubkey;

use crate::{error::OreError, event::OreEvent};

use super::{Market, SwapDirection, TokenType, VirtualLimitOrder};
use crate::event::SwapEvent;

impl Market {
    pub fn buy_exact_out(&mut self, base_out: u64) -> Result<SwapEvent, OreError> {
        // Check if there is enough liquidity.
        if self.base.balance < base_out {
            return Err(OreError::InsufficientLiquidity);
        }

        // Upcast data.
        let base_out = base_out as u128;

        // Get virtual limit order.
        let VirtualLimitOrder {
            size_in_base: ask_size_in_base,
            size_in_quote: ask_size_in_quote,
        } = self.get_virtual_limit_order(SwapDirection::Buy);

        // Execute swap.
        let (base_via_ask, quote_via_ask, base_via_curve, quote_via_curve) =
            if !self.sandwich_resistance_enabled() {
                // Fill entire swap via curve.
                let base_via_curve = base_out;
                let quote_via_curve = self.get_quote_in(base_via_curve)?;
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Buy)?;
                (0, 0, base_via_curve, quote_via_curve)
            } else if ask_size_in_base >= base_out {
                // Fill entire swap through virtual limit order.
                let base_via_ask = base_out;
                let quote_via_ask = self.get_complementary_limit_order_size(
                    base_via_ask,
                    SwapDirection::Buy,
                    TokenType::Base,
                );
                self.update_reserves(base_via_ask, quote_via_ask, SwapDirection::Buy)?;
                (base_via_ask, quote_via_ask, 0, 0)
            } else {
                // Partially fill swap through virtual limit order.
                let base_via_ask = ask_size_in_base;
                let quote_via_ask = ask_size_in_quote;
                self.update_reserves(base_via_ask, quote_via_ask, SwapDirection::Buy)?;

                // Fill remaining swap amount through pool.
                let base_via_curve = base_out - base_via_ask;
                let quote_via_curve = self.get_quote_in(base_via_curve)?;
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Buy)?;
                (base_via_ask, quote_via_ask, base_via_curve, quote_via_curve)
            };

        // Calculate fee.
        let quote_post_fee = quote_via_ask + quote_via_curve;
        let quote_in = self.pre_fee(quote_post_fee as u64) as u128;
        let quote_fee = quote_in - quote_post_fee;

        // Produce swap result.
        let swap_event = SwapEvent {
            disc: OreEvent::Swap as u64,
            authority: Pubkey::default(),
            block_id: 0,
            direction: SwapDirection::Buy as u64,
            base_to_transfer: base_out as u64,
            quote_to_transfer: quote_in as u64,
            base_via_order: base_via_ask as u64,
            quote_via_order: quote_via_ask as u64,
            base_via_curve: base_via_curve as u64,
            quote_via_curve: quote_via_curve as u64,
            quote_fee: quote_fee as u64,
            ts: 0,
        };

        // Sanity check swap result.
        assert!(
            swap_event.base_to_transfer == swap_event.base_via_order + swap_event.base_via_curve
        );
        assert!(
            swap_event.quote_to_transfer
                == swap_event.quote_via_order + swap_event.quote_via_curve + swap_event.quote_fee
        );

        // Return.
        Ok(swap_event)
    }
}
