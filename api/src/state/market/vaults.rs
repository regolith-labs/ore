use solana_program::log::sol_log;
use steel::*;

use crate::error::OreError;

use super::Market;

/// Vault reserve checks.
impl Market {
    /// Sanity check that vaults have reserves for all market debts.
    /// Assumes the token accounts have already been validated as the market's base and quote vaults.
    // pub fn check_vaults(
    //     &self,
    //     base_vault: &TokenAccount,
    //     quote_vault: &TokenAccount,
    // ) -> Result<(), OreError> {
    //     self.check_base_vault(base_vault)?;
    //     self.check_quote_vault(quote_vault)?;
    //     Ok(())
    // }

    /// Sanity check that base vault has reserves for all market debts.
    /// Assumes the token account has already been validated as the market's base vault.
    // pub fn check_base_vault(&self, base_vault: &TokenAccount) -> Result<(), OreError> {
    //     if base_vault.amount() < self.base.balance {
    //         sol_log(&format!("A base_vault.amount: {}", base_vault.amount()));
    //         sol_log(&format!("A self.base.balance: {}", self.base.balance));
    //         sol_log("Insufficient base vault reserves");
    //         return Err(OreError::InsufficientVaultReserves.into());
    //     }
    //     Ok(())
    // }

    /// Sanity check that quote vault has reserves for all market debts.
    /// Assumes the token account has already been validated as the market's quote vault.
    pub fn check_quote_vault(&self, quote_vault: &TokenAccount) -> Result<(), OreError> {
        if quote_vault.amount() < self.quote.balance + self.fee.uncollected {
            sol_log("Insufficient quote vault reserves");
            return Err(OreError::InsufficientVaultReserves.into());
        }
        Ok(())
    }
}
