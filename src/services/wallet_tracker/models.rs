use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// KOL (Key Opinion Leader) wallet information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KolWallet {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    /// Name of the KOL
    pub name: String,
    
    /// Description of the KOL
    pub description: String,
    
    /// Wallet addresses associated with this KOL
    pub wallet_addresses: Vec<String>,
    
    /// Influence score (0.0 - 1.0)
    pub influence_score: f64,
    
    /// Category (Trader, Developer, Influencer, VC, Protocol, Whale)
    pub category: String,
    
    /// Twitter handle (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub twitter_handle: Option<String>,
    
    /// When this record was last updated
    pub last_updated: DateTime<Utc>,
    
    /// Whether to include in analysis
    pub active: bool,
}

/// KOL token ownership information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KolOwnership {
    /// KOL ID reference (ObjectId as string)
    pub kol_id: String,
    
    /// KOL name for display
    pub name: String,
    
    /// Wallet address that holds the token
    pub wallet_address: String,
    
    /// Token amount held
    pub position_size: f64,
    
    /// When the position was first observed
    pub entry_time: DateTime<Utc>,
}

/// Token recommendation with KOL ownership data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRecommendation {
    /// MongoDB ObjectId
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    /// Token contract address
    pub token_address: String,
    
    /// Token symbol
    pub symbol: String,
    
    /// Token name
    pub name: String,
    
    /// Token decimals
    pub decimals: i32,
    
    /// Token logo URI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    
    /// Analysis date
    pub analysis_date: DateTime<Utc>,
    
    /// Overall score (0.0 - 1.0)
    pub overall_score: f64,
    
    /// Market score (0.0 - 1.0)
    pub market_score: f64,
    
    /// Social score (0.0 - 1.0)
    pub social_score: f64,
    
    /// Development score (0.0 - 1.0)
    pub dev_score: f64,
    
    /// Risk score (0.0 - 1.0)
    pub risk_score: f64,
    
    /// Token price in USD
    pub price: f64,
    
    /// Token liquidity in USD
    pub liquidity: f64,
    
    /// Market cap in USD
    pub market_cap: f64,
    
    /// 24-hour volume in USD
    pub volume_24h: f64,
    
    /// Number of token holders
    pub holders: i64,
    
    /// Key strengths of the token
    pub strengths: Vec<String>,
    
    /// Key risks associated with the token
    pub risks: Vec<String>,
    
    /// Final recommendation
    pub recommendation: String,
    
    /// KOL ownership data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kol_ownership: Option<Vec<KolOwnership>>,
    
    /// Decision reasoning output from the LLM
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_reasoning: Option<DecisionReasoning>,
    
    /// Vector embedding for similarity search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

/// Detailed decision reasoning from the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionReasoning {
    /// Market analysis reasoning
    pub market_analysis: String,
    
    /// Sentiment analysis reasoning
    pub sentiment_analysis: String,
    
    /// Social signals analysis
    pub social_signals: String,
    
    /// Risk assessment
    pub risk_assessment: String,
    
    /// Final reasoning and conclusion
    pub final_reasoning: String,
} 