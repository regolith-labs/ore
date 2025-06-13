use steel::Clock;

use crate::{consts::SLOT_WINDOW, error::OreError};

use super::{Market, SwapDirection, TokenType, VirtualLimitOrder};

impl Market {
    /// This function solves the closed-form solution for the size of the virtual limit order
    /// in the pool. The virutal limit order is always priced at the snapshot price.
    ///
    /// The size of the limit order is determined by the following constraint:
    ///
    /// ```no_run
    /// (quote_snapshot / base_snapshot) = (quote_reserves + ∆_quote) / (base_reserves + ∆_base)
    /// ```
    ///
    /// Note that the signs of ∆_quote and ∆_base are always flipped.
    ///
    /// This means that the size of the limit order is set such that the new pool price
    /// after the swap is equal to the price at the snapshot.
    ///
    /// Because we know the limit order is priced at the snapshot price, we can derive
    /// the following equations:
    /// -  ∆_base = -∆_quote * base_snapshot / quote_snapshot
    /// -  ∆_quote = -∆_base * quote_snapshot / base_snapshot
    ///
    ///
    /// We can then solve for ∆_base and ∆_quote after substituting the above equations. There are separate cases
    /// for buy and sell
    ///
    /// - Limit order on the buy side (bid)
    /// ```no_run
    /// ∆_base = (base_snapshot * quote_reserves - quote_snapshot * base_reserves) / (2 * quote_snapshot)
    /// ∆_quote = (base_snapshot * quote_reserves - quote_snapshot * base_reserves) / (2 * base_snapshot)
    /// ```
    ///
    /// - Limit order on the sell side (ask)
    /// ```no_run
    /// ∆_base = (quote_snapshot * base_reserves - base_snapshot * quote_reserves) / (2 * quote_snapshot)
    /// ∆_quote = (quote_snapshot * base_reserves - base_snapshot * quote_reserves) / (2 * base_snapshot)
    /// ```
    pub fn get_virtual_limit_order(&self, direction: SwapDirection) -> VirtualLimitOrder {
        // Upcast data.
        let base_balance = self.base.liquidity();
        let quote_balance = self.quote.liquidity();
        let base_snapshot = self.snapshot.base_balance as u128;
        let quote_snapshot = self.snapshot.quote_balance as u128;

        // Get virtual limit order.
        match direction {
            SwapDirection::Buy => {
                let ask = if quote_snapshot * base_balance > base_snapshot * quote_balance {
                    let size_in_quote = (quote_snapshot * base_balance
                        - base_snapshot * quote_balance)
                        / (2 * base_snapshot);
                    let size_in_base = size_in_quote * base_snapshot / quote_snapshot;
                    VirtualLimitOrder {
                        size_in_base,
                        size_in_quote,
                    }
                } else {
                    VirtualLimitOrder::default()
                };
                ask
            }
            SwapDirection::Sell => {
                let bid = if base_snapshot * quote_balance > quote_snapshot * base_balance {
                    let size_in_base = (base_snapshot * quote_balance
                        - quote_snapshot * base_balance)
                        / (2 * quote_snapshot);
                    let size_in_quote = size_in_base * quote_snapshot / base_snapshot;
                    VirtualLimitOrder {
                        size_in_base,
                        size_in_quote,
                    }
                } else {
                    VirtualLimitOrder::default()
                };
                bid
            }
        }
    }

    /// This function returns the size of the virtual limit order in the complementary token type
    /// given an `amount` and the `input_token_type`.
    ///  - If the `input_token_type` is Base, then the size of the limit order in Quote is computed.
    ///  - If the `input_token_type` is Quote, then the size of the limit order in Base is computed.
    pub(crate) fn get_complementary_limit_order_size(
        &self,
        amount: u128,
        direction: SwapDirection,
        token_type: TokenType,
    ) -> u128 {
        if amount == 0 {
            return 0;
        }
        let quote_snapshot = self.snapshot.quote_balance as u128;
        let base_snapshot = self.snapshot.base_balance as u128;

        match direction {
            SwapDirection::Buy => {
                match token_type {
                    // If `amount` is in base, then the size of the limit order in quote is computed and rounded up
                    TokenType::Base => ((amount * quote_snapshot).saturating_sub(1)
                        / base_snapshot)
                        .saturating_add(1),
                    // If `amount` is in quote, then the size of the limit order in base is computed
                    TokenType::Quote => amount * base_snapshot / quote_snapshot,
                }
            }
            SwapDirection::Sell => {
                match token_type {
                    // If `amount` is in base, then the size of the limit order in quote is computed
                    TokenType::Base => amount * quote_snapshot / base_snapshot,
                    // If `amount` is in quote, then the size of the limit order in base is computed and rounded up
                    TokenType::Quote => ((amount * base_snapshot).saturating_sub(1)
                        / quote_snapshot)
                        .saturating_add(1),
                }
            }
        }
    }

    pub(crate) fn update_snapshot(&mut self, clock: &Clock) {
        let slot = clock.slot;
        let snapshot_slot = (slot / SLOT_WINDOW) * SLOT_WINDOW;
        if snapshot_slot != self.snapshot.slot {
            self.snapshot.slot = snapshot_slot;
            self.snapshot.base_balance = self.base.liquidity() as u64;
            self.snapshot.quote_balance = self.quote.liquidity() as u64;
        }
    }

    pub(crate) fn update_reserves(
        &mut self,
        base: u128,
        quote: u128,
        direction: SwapDirection,
    ) -> Result<(), OreError> {
        match direction {
            SwapDirection::Buy => {
                if base > self.base.balance as u128 {
                    return Err(OreError::InsufficientVaultReserves.into());
                }
                self.base.balance -= base as u64;
                self.quote.balance += quote as u64;
            }
            SwapDirection::Sell => {
                if quote > self.quote.balance as u128 {
                    return Err(OreError::InsufficientVaultReserves.into());
                }
                self.base.balance += base as u64;
                self.quote.balance -= quote as u64;
            }
        }
        Ok(())
    }
}
