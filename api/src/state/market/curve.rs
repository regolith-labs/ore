use crate::error::OreError;

use super::Market;

// TODO Add weights.

impl Market {
    /// Returns the constant product invariant.
    pub(crate) fn k(&self) -> u128 {
        let base_reserves = self.base.balance as u128;
        let quote_reserves = self.quote.balance as u128;
        (base_reserves * quote_reserves).saturating_sub(1)
    }

    /// Returns the amount of base tokens that can be bought from a given amount of quote tokens.
    pub fn get_base_out(&self, quote_in: u128) -> u128 {
        let base_reserves = self.base.balance as u128;
        let quote_reserves = self.quote.balance as u128;
        let base_out = base_reserves - (self.k() / (quote_reserves + quote_in)).saturating_add(1);
        base_out
    }

    /// Returns the amount of quote tokens received from selling a given amount of base tokens.
    pub fn get_quote_out(&self, base_in: u128) -> u128 {
        let base_reserves = self.base.balance as u128;
        let quote_reserves = self.quote.balance as u128;
        let quote_out = quote_reserves - (self.k() / (base_reserves + base_in)).saturating_add(1);
        quote_out
    }

    /// Returns the amount of quote tokens needed to buy a given amount of base tokens.
    pub fn get_quote_in(&self, base_out: u128) -> Result<u128, OreError> {
        let base_reserves = self.base.balance as u128;
        let quote_reserves = self.quote.balance as u128;
        if base_out >= base_reserves {
            return Err(OreError::InsufficientVaultReserves.into());
        }
        let quote_in = (self.k() / (base_reserves - base_out)).saturating_add(1) - quote_reserves;
        Ok(quote_in)
    }

    /// Returns the amount of base tokens which must be sold to receive a given amount of quote tokens.
    pub fn get_base_in(&self, quote_out: u128) -> Result<u128, OreError> {
        let base_reserves = self.base.balance as u128;
        let quote_reserves = self.quote.balance as u128;
        if quote_out >= quote_reserves {
            return Err(OreError::InsufficientVaultReserves.into());
        }
        let base_in = (self.k() / (quote_reserves - quote_out)).saturating_add(1) - base_reserves;
        Ok(base_in)
    }
}
