use spl_associated_token_account::get_associated_token_address;
use steel::*;

use crate::state::{market_pda, OreAccount};

// TODO Bonding curve stuff

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Market {
    /// Base token parameters.
    pub base: TokenParams,

    /// Quote token parameters.
    pub quote: TokenParams,

    /// Fee parameters.
    pub fee: FeeParams,

    /// Snapshot of the market state at the time of the last swap.
    pub snapshot: Snapshot,

    /// The id of the current block.
    pub block_id: u64,
}

impl Market {
    pub fn pda(&self) -> (Pubkey, u8) {
        market_pda()
    }

    pub fn base_vault(&self) -> Pubkey {
        get_associated_token_address(&self.pda().0, &self.base.mint)
    }

    pub fn quote_vault(&self) -> Pubkey {
        get_associated_token_address(&self.pda().0, &self.quote.mint)
    }

    pub fn sandwich_resistance_enabled(&self) -> bool {
        self.snapshot.enabled == 1
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct TokenParams {
    /// Mint of the token.
    pub mint: Pubkey,

    /// Amount of tokens held in liquidity.
    pub balance: u64,

    /// Amount of virtual tokens held in liquidity.
    pub balance_virtual: u64,
}

impl TokenParams {
    pub fn liquidity(&self) -> u128 {
        (self.balance + self.balance_virtual) as u128
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct FeeParams {
    /// Cumulative protocol fees.
    pub cumulative: u64,

    /// Fee rate in basis points.
    pub rate: u64,

    /// Current uncollected protocol fees.
    pub uncollected: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Snapshot {
    /// Whether sandwich resistance is enabled.
    pub enabled: u64,

    /// Base token balance at the time of the snapshot.
    pub base_balance: u64,

    /// Quote token balance at the time of the snapshot.
    pub quote_balance: u64,

    /// Slot at which the snapshot was taken.
    pub slot: u64,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct VirtualLimitOrder {
    /// Size of the virtual limit order in base tokens.
    pub size_in_base: u128,

    /// Size of the virtual limit order in quote tokens.
    pub size_in_quote: u128,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum SwapDirection {
    /// Swap quote tokens for base tokens.
    Buy = 0,

    /// Swap base tokens for quote tokens.
    Sell = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum SwapPrecision {
    /// Swap with precision exact in amount.
    ExactIn = 0,

    /// Swap with precision exact out amount.
    ExactOut = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum TokenType {
    /// Base token.
    Base = 0,

    /// Quote token.
    Quote = 1,
}

account!(OreAccount, Market);

#[cfg(test)]
mod tests {
    use crate::consts::FEE_RATE_BPS;

    use super::*;

    #[test]
    fn test_fees_buy_exact_in() {
        let mut market = new_market();
        let swap = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                Clock::default(),
            )
            .unwrap();
        assert_eq!(swap.quote_via_curve, 99_000);
        assert_eq!(market.fee.uncollected, 1000); // Protocol fee is 1%
    }

    #[test]
    fn test_fees_sell_exact_in() {
        let mut market = new_market();
        let swap = market
            .swap(
                100_000,
                SwapDirection::Sell,
                SwapPrecision::ExactIn,
                Clock::default(),
            )
            .unwrap();
        assert_eq!(swap.quote_via_curve, 98_991);
        assert_eq!(market.fee.uncollected, 999);
    }

    #[test]
    fn test_fees_buy_exact_out() {
        let mut market = new_market();
        let swap = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactOut,
                Clock::default(),
            )
            .unwrap();
        assert_eq!(swap.quote_via_curve, 100_011);
        assert_eq!(market.fee.uncollected, 1010);
    }

    #[test]
    fn test_fees_sell_exact_out() {
        let mut market = new_market();
        let swap = market
            .swap(
                100_000,
                SwapDirection::Sell,
                SwapPrecision::ExactOut,
                Clock::default(),
            )
            .unwrap();
        assert_eq!(swap.quote_via_curve, 101_010);
        assert_eq!(market.fee.uncollected, 1010);
    }

    #[test]
    fn test_fills() {
        let mut market = new_market();
        let mut clock = Clock::default();
        clock.slot = 10;

        // Small buy
        // Assert swap is filled via curve.
        // Post swap, price is above snapshot.
        let swap_1 = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_1.base_via_curve > 0 && swap_1.quote_via_curve > 0);
        assert!(swap_1.base_via_order == 0 && swap_1.quote_via_order == 0);

        // Large sell
        // Assert swap is partially filled via order and partially filled via curve.
        // Post swap, price is below snapshot.
        let swap_2 = market
            .swap(
                1_000_000,
                SwapDirection::Sell,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_2.base_via_curve > 0 && swap_2.quote_via_curve > 0);
        assert!(swap_2.base_via_order > 0 && swap_2.quote_via_order > 0);

        // Small buy
        // Assert swap is filled via order
        // Post swap, price is still below snapshot.
        let swap_3 = market
            .swap(
                1_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_3.base_via_curve == 0 && swap_3.quote_via_curve == 0);
        assert!(swap_3.base_via_order > 0 && swap_3.quote_via_order > 0);

        // Large buy
        // Assert swap is partially filled via order and partially filled via curve.
        // Post swap, price is above snapshot.
        let swap_4 = market
            .swap(
                1_000_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_4.base_via_curve > 0 && swap_4.quote_via_curve > 0);
        assert!(swap_4.base_via_order > 0 && swap_4.quote_via_order > 0);
    }

    #[test]
    fn test_sandwich() {
        let mut market = new_market();
        market.fee.rate = 0;
        market.snapshot = Snapshot {
            enabled: 0,
            base_balance: 0,
            quote_balance: 0,
            slot: 0,
        };

        let mut clock = Clock::default();
        clock.slot = 10;

        // Open sandwich
        let swap_1 = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        let amount_base_1 = swap_1.base_to_transfer;
        assert!(swap_1.base_via_curve > 0 && swap_1.quote_via_curve > 0);
        assert!(swap_1.base_via_order == 0 && swap_1.quote_via_order == 0);

        // Victim buys
        let swap_2 = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_2.base_via_curve > 0 && swap_2.quote_via_curve > 0);
        assert!(swap_2.base_via_order == 0 && swap_2.quote_via_order == 0);

        // Close sandwich
        let swap_3 = market
            .swap(
                amount_base_1,
                SwapDirection::Sell,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_3.base_via_curve > 0 && swap_3.quote_via_curve > 0);
        assert!(swap_3.base_via_order == 0 && swap_3.quote_via_order == 0);

        // Assert sandwich attack succeeded
        assert!(swap_3.quote_to_transfer > swap_1.quote_to_transfer);
    }

    #[test]
    fn test_sandwich_resistance() {
        let mut market = new_market();
        market.fee.rate = 0;

        let mut clock = Clock::default();
        clock.slot = 10;

        // Open sandwich
        let swap_1 = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        let amount_base_1 = swap_1.base_to_transfer;
        assert!(swap_1.base_via_curve > 0 && swap_1.quote_via_curve > 0);
        assert!(swap_1.base_via_order == 0 && swap_1.quote_via_order == 0);

        // Victim buys
        let swap_2 = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_2.base_via_curve > 0 && swap_2.quote_via_curve > 0);
        assert!(swap_2.base_via_order == 0 && swap_2.quote_via_order == 0);

        // Close sandwich
        let swap_3 = market
            .swap(
                amount_base_1,
                SwapDirection::Sell,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap_3.base_via_curve == 0 && swap_3.quote_via_curve == 0);
        assert!(swap_3.base_via_order > 0 && swap_3.quote_via_order > 0);

        // Assert sandwich attack failed
        assert!(swap_3.quote_to_transfer <= swap_1.quote_to_transfer);
    }

    #[test]
    fn test_virtual_liquidity() {
        let mut market = new_market();
        market.fee.rate = 0;
        market.quote.balance_virtual = 1_000_000_000;
        market.quote.balance = 0;

        let mut clock = Clock::default();
        clock.slot = 10;

        // Sell
        // Assert swap fails without real liquidity to satisfy order.
        let swap = market.swap(
            100_000,
            SwapDirection::Sell,
            SwapPrecision::ExactIn,
            clock.clone(),
        );
        assert!(swap.is_err());

        // Buy
        // Assert buy succeeds adding liquidity.
        let swap = market
            .swap(
                100_000,
                SwapDirection::Buy,
                SwapPrecision::ExactIn,
                clock.clone(),
            )
            .unwrap();
        assert!(swap.base_via_curve > 0 && swap.quote_via_curve > 0);
        assert!(swap.base_via_order == 0 && swap.quote_via_order == 0);
        assert_eq!(market.quote.balance, 100_000);
        assert_eq!(market.quote.balance_virtual, 1_000_000_000);

        // Sell
        // Assert sell fails if there is insufficient liquidity.
        let swap = market.swap(
            100_001,
            SwapDirection::Sell,
            SwapPrecision::ExactOut,
            clock.clone(),
        );
        assert!(swap.is_err());

        // Sell
        // Assert sell succeeds removing liquidity.
        let swap = market
            .swap(
                100_000,
                SwapDirection::Sell,
                SwapPrecision::ExactOut,
                clock.clone(),
            )
            .unwrap();
        assert!(swap.base_via_curve > 0 && swap.quote_via_curve > 0);
        assert!(swap.base_via_order > 0 && swap.quote_via_order > 0);
        assert_eq!(market.quote.balance, 0);
        assert_eq!(market.quote.balance_virtual, 1_000_000_000);
    }

    fn new_market() -> Market {
        Market {
            base: TokenParams {
                mint: Pubkey::new_unique(),
                balance: 1_000_000_000,
                balance_virtual: 0,
            },
            quote: TokenParams {
                mint: Pubkey::new_unique(),
                balance: 1_000_000_000,
                balance_virtual: 0,
            },
            fee: FeeParams {
                cumulative: 0,
                uncollected: 0,
                rate: FEE_RATE_BPS,
            },
            snapshot: Snapshot {
                enabled: 1,
                base_balance: 0,
                quote_balance: 0,
                slot: 0,
            },
            block_id: 0,
        }
    }
}
