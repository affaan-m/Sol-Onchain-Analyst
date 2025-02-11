use super::SolanaAgentKit;
use crate::models::market_signal::{MarketSignal, SignalType};
use crate::utils::{decimal_to_f64, f64_to_decimal};
use anyhow::Result;
use tracing::{info, warn};

pub struct TradingEngine {
    min_confidence: f64,
    max_trade_size: f64,
    agent: SolanaAgentKit,
}

#[derive(Debug)]
pub struct TradeDecision {
    pub action: String,
    pub symbol: String,
    pub amount: f64,
    pub reason: String,
    pub confidence: f64,
    pub mint_address: Option<String>,
}

impl TradingEngine {
    pub fn new(min_confidence: f64, max_trade_size: f64, agent: SolanaAgentKit) -> Self {
        Self {
            min_confidence,
            max_trade_size,
            agent,
        }
    }

    pub async fn execute_trade(&self, signal: &MarketSignal) -> Result<String> {
        let min_conf = f64_to_decimal(self.min_confidence);

        if signal.confidence < min_conf {
            warn!("Signal confidence too low for trading");
            return Ok("Signal confidence too low".to_string());
        }

        let max_size = f64_to_decimal(self.max_trade_size);
        let _amount = decimal_to_f64(&(max_size.clone() * signal.confidence.clone()).min(max_size));

        let action = match signal.signal_type {
            SignalType::Buy
            | SignalType::StrongBuy
            | SignalType::PriceSpike
            | SignalType::VolumeSurge => "BUY",
            SignalType::Sell | SignalType::StrongSell | SignalType::PriceDrop => "SELL",
            SignalType::Hold => "HOLD",
        };

        info!(
            "Executing {} trade for {} with confidence {:.2}",
            action,
            signal.asset_address,
            decimal_to_f64(&signal.confidence)
        );

        // TODO: Implement actual Solana transaction execution
        // For now, just return a mock signature
        Ok(format!(
            "mock_tx_{}_{}",
            action.to_lowercase(),
            signal.asset_address
        ))
    }

    pub fn get_min_confidence(&self) -> f64 {
        self.min_confidence
    }

    pub fn get_max_trade_size(&self) -> f64 {
        self.max_trade_size
    }
}
