use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde::{Deserialize, Serialize};
use crate::types::error::BirdeyeError;

const WEBSOCKET_URL: &str = "wss://public-api.birdeye.so/socket";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketUpdate {
    pub address: String,
    pub price: f64,
    pub volume_24h: f64,
    pub price_change_24h: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeUpdate {
    pub address: String,
    pub price: f64,
    pub size: f64,
    pub side: TradeSide,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TradeSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubscribeMessage {
    action: String,
    token: String,
    api_key: String,
}

#[derive(Debug)]
pub struct WebSocketProvider {
    api_key: String,
    market_sender: broadcast::Sender<MarketUpdate>,
    trade_sender: broadcast::Sender<TradeUpdate>,
}

impl WebSocketProvider {
    pub fn new(api_key: &str) -> Self {
        let (market_sender, _) = broadcast::channel(1000);
        let (trade_sender, _) = broadcast::channel(1000);
        
        Self {
            api_key: api_key.to_string(),
            market_sender,
            trade_sender,
        }
    }

    pub fn subscribe_market_updates(&self) -> broadcast::Receiver<MarketUpdate> {
        self.market_sender.subscribe()
    }

    pub fn subscribe_trade_updates(&self) -> broadcast::Receiver<TradeUpdate> {
        self.trade_sender.subscribe()
    }

    pub async fn connect_and_stream(&self, tokens: Vec<String>) -> Result<(), BirdeyeError> {
        let url = format!("{}?apiKey={}", WEBSOCKET_URL, self.api_key);
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| BirdeyeError::WebSocketError(e.to_string()))?;
        
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to tokens
        for token in tokens {
            let subscribe_msg = SubscribeMessage {
                action: "subscribe".to_string(),
                token,
                api_key: self.api_key.clone(),
            };
            
            let msg = serde_json::to_string(&subscribe_msg)
                .map_err(|e| BirdeyeError::SerializationError(e.to_string()))?;
            
            write.send(Message::Text(msg))
                .await
                .map_err(|e| BirdeyeError::WebSocketError(e.to_string()))?;
        }

        let market_sender = self.market_sender.clone();
        let trade_sender = self.trade_sender.clone();

        // Spawn message handling task
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(market_update) = serde_json::from_str::<MarketUpdate>(&text) {
                            let _ = market_sender.send(market_update);
                        } else if let Ok(trade_update) = serde_json::from_str::<TradeUpdate>(&text) {
                            let _ = trade_sender.send(trade_update);
                        }
                    }
                    Ok(Message::Close(_)) => break,
                    Err(e) => {
                        eprintln!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }
}

// Example usage:
//
// let ws_provider = WebSocketProvider::new("your-api-key");
// 
// // Subscribe to market updates
// let mut market_rx = ws_provider.subscribe_market_updates();
// tokio::spawn(async move {
//     while let Ok(update) = market_rx.recv().await {
//         println!("Market update: {:?}", update);
//     }
// });
//
// // Subscribe to trade updates
// let mut trade_rx = ws_provider.subscribe_trade_updates();
// tokio::spawn(async move {
//     while let Ok(update) = trade_rx.recv().await {
//         println!("Trade update: {:?}", update);
//     }
// });
//
// // Connect and start streaming
// ws_provider.connect_and_stream(vec!["token-address-1".to_string(), "token-address-2".to_string()]).await?;
