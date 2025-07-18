use steel::Pubkey;

use crate::{error::OreError, event::OreEvent};

use super::{Market, SwapDirection, TokenType, VirtualLimitOrder};
use crate::event::SwapEvent;

impl Market {
    pub fn sell_exact_out(&mut self, quote_out: u64) -> Result<SwapEvent, OreError> {
        // Check if there is enough liquidity.
        if self.quote.balance < quote_out {
            return Err(OreError::InsufficientLiquidity);
        }

        // Calculate fee.
        let quote_out_pre_fee = self.pre_fee(quote_out) as u128;
        let quote_fee = quote_out_pre_fee - quote_out as u128;

        // Upcast data.
        let quote_out = quote_out as u128;

        // Get virtual limit order.
        let VirtualLimitOrder {
            size_in_base: bid_size_in_base,
            size_in_quote: bid_size_in_quote,
        } = self.get_virtual_limit_order(SwapDirection::Sell);

        // Execute swap.
        let (base_via_bid, quote_via_bid, base_via_curve, quote_via_curve) =
            if !self.sandwich_resistance_enabled() {
                // Fill entire swap via curve.
                let quote_via_curve = quote_out_pre_fee;
                let base_via_curve = self.get_base_in(quote_via_curve)?;
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Sell)?;
                (0, 0, base_via_curve, quote_via_curve)
            } else if bid_size_in_quote >= quote_out {
                // Fill entire swap through virtual limit order.
                let quote_via_bid = quote_out_pre_fee;
                let base_via_bid = self.get_complementary_limit_order_size(
                    quote_via_bid,
                    SwapDirection::Sell,
                    TokenType::Quote,
                );
                self.update_reserves(base_via_bid, quote_via_bid, SwapDirection::Sell)?;
                (base_via_bid, quote_via_bid, 0, 0)
            } else {
                // Partially fill swap through virtual limit order.
                let base_via_bid = bid_size_in_base;
                let quote_via_bid = bid_size_in_quote;
                self.update_reserves(base_via_bid, quote_via_bid, SwapDirection::Sell)?;

                // Fill remaining swap amount through pool.
                let quote_via_curve = quote_out_pre_fee - quote_via_bid;
                let base_via_curve = self.get_base_in(quote_via_curve)?;
                self.update_reserves(base_via_curve, quote_via_curve, SwapDirection::Sell)?;
                (base_via_bid, quote_via_bid, base_via_curve, quote_via_curve)
            };

        // Calculate fee.
        let base_in = base_via_bid + base_via_curve;

        // Produce swap result.
        let swap_event = SwapEvent {
            disc: OreEvent::Swap as u64,
            authority: Pubkey::default(),
            block_id: 0,
            direction: SwapDirection::Sell as u64,
            base_to_transfer: base_in as u64,
            quote_to_transfer: quote_out as u64,
            base_via_order: base_via_bid as u64,
            quote_via_order: quote_via_bid as u64,
            base_via_curve: base_via_curve as u64,
            quote_via_curve: quote_via_curve as u64,
            quote_fee: quote_fee as u64,
            base_liquidity: self.base.liquidity() as u64,
            quote_liquidity: self.quote.liquidity() as u64,
            miner_hashpower: 0,
            block_hashpower: 0,
            ts: 0,
        };

        // Sanity check swap result.
        assert!(
            swap_event.base_to_transfer == swap_event.base_via_order + swap_event.base_via_curve
        );
        assert!(
            swap_event.quote_to_transfer
                == swap_event.quote_via_order + swap_event.quote_via_curve - swap_event.quote_fee
        );

        // Return.
        Ok(swap_event)
    }
}
