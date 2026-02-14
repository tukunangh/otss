//! OTSS Core - Risk management module
//!
//! Provides risk checks and position limit enforcement.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::portfolio::Portfolio;
use crate::types::{Money, Price, Quantity, Symbol};

/// Risk configuration for trading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Maximum position size per symbol (in number of shares/contracts)
    pub max_position_size: Quantity,
    /// Maximum position value per symbol
    pub max_position_value: Money,
    /// Maximum portfolio exposure percentage (0.0 - 1.0)
    pub max_portfolio_exposure: Decimal,
    /// Maximum daily loss before trading is halted
    pub max_daily_loss: Money,
    /// Maximum single trade size
    pub max_single_order_size: Quantity,
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            max_position_size: Quantity::from(10000), // Default 10k shares
            max_position_value: Money::from_str("1000000.00").unwrap_or_else(|_| {
                Money::from_decimal(Decimal::from_str("1000000.00").unwrap())
            }),
            max_portfolio_exposure: Decimal::from_str("0.95").unwrap_or_else(|_| Decimal::ONE),
            max_daily_loss: Money::from_str("100000.00").unwrap_or_else(|_| {
                Money::from_decimal(Decimal::from_str("100000.00").unwrap())
            }),
            max_single_order_size: Quantity::from(5000),
        }
    }
}

/// The result of a risk check
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskResult {
    /// Check passed, order is accepted
    Accepted,
    /// Check failed, order is rejected with reason
    Rejected { reason: String },
    /// Check passed with a warning
    Warning { message: String },
}

impl RiskResult {
    /// Check if the result is accepted
    pub fn is_accepted(&self) -> bool {
        matches!(self, RiskResult::Accepted) || matches!(self, RiskResult::Warning { .. })
    }

    /// Check if the result is rejected
    pub fn is_rejected(&self) -> bool {
        matches!(self, RiskResult::Rejected { .. })
    }

    /// Get the reason if rejected
    pub fn rejection_reason(&self) -> Option<&str> {
        match self {
            RiskResult::Rejected { reason } => Some(reason),
            _ => None,
        }
    }
}

/// Performs risk checks on orders and positions
#[derive(Debug, Clone)]
pub struct RiskCheck {
    config: RiskConfig,
    daily_pnl: Money,
}

impl RiskCheck {
    /// Create a new risk checker with the given configuration
    pub fn new(config: RiskConfig) -> Self {
        Self {
            config,
            daily_pnl: Money::from_decimal(Decimal::ZERO),
        }
    }

    /// Check if an order size is within limits
    pub fn check_order_size(&self, quantity: Quantity) -> RiskResult {
        let max = self.config.max_single_order_size;
        if quantity.abs().to_decimal() > max.to_decimal() {
            return RiskResult::Rejected {
                reason: format!(
                    "Order size {} exceeds maximum allowed {}",
                    quantity, max
                ),
            };
        }
        RiskResult::Accepted
    }

    /// Check position limits
    pub fn check_position_limits(
        &self,
        symbol: &Symbol,
        new_quantity: Quantity,
        portfolio: &Portfolio,
    ) -> RiskResult {
        // Check max position size
        if new_quantity.abs().to_decimal() > self.config.max_position_size.to_decimal() {
            return RiskResult::Rejected {
                reason: format!(
                    "Position size {} for {} exceeds maximum {}",
                    new_quantity,
                    symbol,
                    self.config.max_position_size
                ),
            };
        }

        // Check position value limits
        if let Some(position) = portfolio.position(symbol) {
            if let Some(market_value) = position.market_value() {
                if market_value.to_decimal() > self.config.max_position_value.to_decimal() {
                    return RiskResult::Rejected {
                        reason: format!(
                            "Position value {} exceeds maximum {}",
                            market_value, self.config.max_position_value
                        ),
                    };
                }
            }
        }

        RiskResult::Accepted
    }

    /// Check portfolio exposure
    pub fn check_portfolio_exposure(&self, portfolio: &Portfolio) -> RiskResult {
        let total_equity = portfolio.total_equity();
        let positions_value = portfolio.positions_value();

        if total_equity.to_decimal().is_zero() {
            return RiskResult::Accepted;
        }

        let exposure = positions_value.to_decimal() / total_equity.to_decimal();

        if exposure > self.config.max_portfolio_exposure {
            RiskResult::Rejected {
                reason: format!(
                    "Portfolio exposure {:.2}% exceeds maximum {:.2}%",
                    exposure * Decimal::from(100),
                    self.config.max_portfolio_exposure * Decimal::from(100)
                ),
            }
        } else if exposure > self.config.max_portfolio_exposure * Decimal::from_str("0.8").unwrap() {
            RiskResult::Warning {
                message: format!(
                    "Portfolio exposure high at {:.2}%",
                    exposure * Decimal::from(100)
                ),
            }
        } else {
            RiskResult::Accepted
        }
    }

    /// Check if daily loss limits have been breached
    pub fn check_daily_loss(&self) -> RiskResult {
        if self.daily_pnl.to_decimal() < -self.config.max_daily_loss.to_decimal() {
            return RiskResult::Rejected {
                reason: format!(
                    "Daily loss limit of {} exceeded. Current P&L: {}",
                    self.config.max_daily_loss, self.daily_pnl
                ),
            };
        }
        RiskResult::Accepted
    }

    /// Update daily P&L
    pub fn update_daily_pnl(&mut self, pnl: Money) {
        // This would typically add to an existing daily P&L value
        let current = self.daily_pnl.to_decimal();
        let new_pnl = current + pnl.to_decimal();
        self.daily_pnl = Money::from_decimal(new_pnl);
    }

    /// Get current daily P&L
    pub fn daily_pnl(&self) -> Money {
        self.daily_pnl
    }

    /// Perform a full risk check
    pub fn check_all(
        &self,
        symbol: &Symbol,
        quantity: Quantity,
        portfolio: &Portfolio,
    ) -> RiskResult {
        // Run all checks in sequence
        let checks = [
            self.check_daily_loss(),
            self.check_order_size(quantity),
            self.check_position_limits(symbol, quantity, portfolio),
            self.check_portfolio_exposure(portfolio),
        ];

        for check in &checks {
            if check.is_rejected() {
                return check.clone();
            }
        }

        // Return the most severe non-accepted result, or Accepted
        for check in &checks {
            if matches!(check, RiskResult::Warning { .. }) {
                return check.clone();
            }
        }

        RiskResult::Accepted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_risk_config_default() {
        let config = RiskConfig::default();
        assert_eq!(config.max_single_order_size.to_string(), "5000");
    }

    #[test]
    fn test_check_order_size() {
        let config = RiskConfig::default();
        let risk = RiskCheck::new(config);

        let result = risk.check_order_size(Quantity::from(1000));
        assert!(result.is_accepted());

        let result = risk.check_order_size(Quantity::from(10000));
        assert!(!result.is_accepted());
    }

    #[test]
    fn test_risk_result_types() {
        assert!(RiskResult::Accepted.is_accepted());
        assert!(!RiskResult::Accepted.is_rejected());

        let rejected = RiskResult::Rejected {
            reason: "Test".to_string(),
        };
        assert!(!rejected.is_accepted());
        assert!(rejected.is_rejected());
    }
}
