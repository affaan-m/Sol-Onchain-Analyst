use crate::models::token_info::TokenExtensions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: T,
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenMarketResponse {
    pub address: String,
    pub decimals: i32,
    pub symbol: String,
    pub name: String,
    #[serde(rename = "marketCap")]
    pub market_cap: Option<f64>,
    pub fdv: Option<f64>,
    pub extensions: Option<TokenExtensions>,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
    pub liquidity: Option<f64>,
    #[serde(rename = "lastTradeUnixTime")]
    pub last_trade_unix_time: Option<i64>,
    #[serde(rename = "lastTradeHumanTime")]
    pub last_trade_human_time: Option<String>,
    pub price: Option<f64>,
    #[serde(rename = "history30mPrice")]
    pub history30m_price: Option<f64>,
    #[serde(rename = "priceChange30mPercent")]
    pub price_change_30m_percent: Option<f64>,
    #[serde(rename = "history1hPrice")]
    pub history1h_price: Option<f64>,
    #[serde(rename = "priceChange1hPercent")]
    pub price_change_1h_percent: Option<f64>,
    #[serde(rename = "history2hPrice")]
    pub history2h_price: Option<f64>,
    #[serde(rename = "priceChange2hPercent")]
    pub price_change_2h_percent: Option<f64>,
    #[serde(rename = "history4hPrice")]
    pub history4h_price: Option<f64>,
    #[serde(rename = "priceChange4hPercent")]
    pub price_change_4h_percent: Option<f64>,
    #[serde(rename = "history6hPrice")]
    pub history6h_price: Option<f64>,
    #[serde(rename = "priceChange6hPercent")]
    pub price_change_6h_percent: Option<f64>,
    #[serde(rename = "history8hPrice")]
    pub history8h_price: Option<f64>,
    #[serde(rename = "priceChange8hPercent")]
    pub price_change_8h_percent: Option<f64>,
    #[serde(rename = "history12hPrice")]
    pub history12h_price: Option<f64>,
    #[serde(rename = "priceChange12hPercent")]
    pub price_change_12h_percent: Option<f64>,
    #[serde(rename = "history24hPrice")]
    pub history24h_price: Option<f64>,
    #[serde(rename = "priceChange24hPercent")]
    pub price_change_24h_percent: Option<f64>,
    #[serde(rename = "uniqueWallet30m")]
    pub unique_wallet30m: Option<i64>,
    #[serde(rename = "uniqueWalletHistory30m")]
    pub unique_wallet_history30m: Option<i64>,
    #[serde(rename = "uniqueWallet30mChangePercent")]
    pub unique_wallet30m_change_percent: Option<f64>,
    #[serde(rename = "uniqueWallet1h")]
    pub unique_wallet1h: Option<i64>,
    #[serde(rename = "uniqueWalletHistory1h")]
    pub unique_wallet_history1h: Option<i64>,
    #[serde(rename = "uniqueWallet1hChangePercent")]
    pub unique_wallet1h_change_percent: Option<f64>,
    #[serde(rename = "uniqueWallet2h")]
    pub unique_wallet2h: Option<i64>,
    #[serde(rename = "uniqueWalletHistory2h")]
    pub unique_wallet_history2h: Option<i64>,
    #[serde(rename = "uniqueWallet2hChangePercent")]
    pub unique_wallet2h_change_percent: Option<f64>,
    #[serde(rename = "uniqueWallet4h")]
    pub unique_wallet4h: Option<i64>,
    #[serde(rename = "uniqueWalletHistory4h")]
    pub unique_wallet_history4h: Option<i64>,
    #[serde(rename = "uniqueWallet4hChangePercent")]
    pub unique_wallet4h_change_percent: Option<f64>,
    #[serde(rename = "uniqueWallet8h")]
    pub unique_wallet8h: Option<i64>,
    #[serde(rename = "uniqueWalletHistory8h")]
    pub unique_wallet_history8h: Option<i64>,
    #[serde(rename = "uniqueWallet8hChangePercent")]
    pub unique_wallet8h_change_percent: Option<f64>,
    #[serde(rename = "uniqueWallet24h")]
    pub unique_wallet24h: Option<i64>,
    #[serde(rename = "uniqueWalletHistory24h")]
    pub unique_wallet_history24h: Option<i64>,
    #[serde(rename = "uniqueWallet24hChangePercent")]
    pub unique_wallet24h_change_percent: Option<f64>,
    pub supply: Option<f64>,
    #[serde(rename = "totalSupply")]
    pub total_supply: Option<f64>,
    pub mc: Option<f64>,
    #[serde(rename = "circulatingSupply")]
    pub circulating_supply: Option<f64>,
    #[serde(rename = "realMc")]
    pub real_mc: Option<f64>,
    pub holder: Option<i64>,
    pub trade30m: Option<i64>,
    #[serde(rename = "tradeHistory30m")]
    pub trade_history30m: Option<i64>,
    #[serde(rename = "trade30mChangePercent")]
    pub trade30m_change_percent: Option<f64>,
    pub sell30m: Option<i64>,
    #[serde(rename = "sellHistory30m")]
    pub sell_history30m: Option<i64>,
    #[serde(rename = "sell30mChangePercent")]
    pub sell30m_change_percent: Option<f64>,
    pub buy30m: Option<i64>,
    #[serde(rename = "buyHistory30m")]
    pub buy_history30m: Option<i64>,
    #[serde(rename = "buy30mChangePercent")]
    pub buy30m_change_percent: Option<f64>,
    pub v30m: Option<f64>,
    #[serde(rename = "v30mUSD")]
    pub v30m_usd: Option<f64>,
    #[serde(rename = "vHistory30m")]
    pub v_history30m: Option<f64>,
    #[serde(rename = "vHistory30mUSD")]
    pub v_history30m_usd: Option<f64>,
    #[serde(rename = "v30mChangePercent")]
    pub v30m_change_percent: Option<f64>,
    #[serde(rename = "vBuy30m")]
    pub v_buy30m: Option<f64>,
    #[serde(rename = "vBuy30mUSD")]
    pub v_buy30m_usd: Option<f64>,
    #[serde(rename = "vBuyHistory30m")]
    pub v_buy_history30m: Option<f64>,
    #[serde(rename = "vBuyHistory30mUSD")]
    pub v_buy_history30m_usd: Option<f64>,
    #[serde(rename = "vBuy30mChangePercent")]
    pub v_buy30m_change_percent: Option<f64>,
    #[serde(rename = "vSell30m")]
    pub v_sell30m: Option<f64>,
    #[serde(rename = "vSell30mUSD")]
    pub v_sell30m_usd: Option<f64>,
    #[serde(rename = "vSellHistory30m")]
    pub v_sell_history30m: Option<f64>,
    #[serde(rename = "vSellHistory30mUSD")]
    pub v_sell_history30m_usd: Option<f64>,
    #[serde(rename = "vSell30mChangePercent")]
    pub v_sell30m_change_percent: Option<f64>,
    pub trade24h: Option<i64>,
    #[serde(rename = "tradeHistory24h")]
    pub trade_history24h: Option<i64>,
    #[serde(rename = "trade24hChangePercent")]
    pub trade24h_change_percent: Option<f64>,
    pub sell24h: Option<i64>,
    #[serde(rename = "sellHistory24h")]
    pub sell_history24h: Option<i64>,
    #[serde(rename = "sell24hChangePercent")]
    pub sell24h_change_percent: Option<f64>,
    pub buy24h: Option<i64>,
    #[serde(rename = "buyHistory24h")]
    pub buy_history24h: Option<i64>,
    #[serde(rename = "buy24hChangePercent")]
    pub buy24h_change_percent: Option<f64>,
    pub v24h: Option<f64>,
    #[serde(rename = "v24hUSD")]
    pub v24h_usd: Option<f64>,
    #[serde(rename = "vHistory24h")]
    pub v_history24h: Option<f64>,
    #[serde(rename = "vHistory24hUSD")]
    pub v_history24h_usd: Option<f64>,
    #[serde(rename = "v24hChangePercent")]
    pub v24h_change_percent: Option<f64>,
    #[serde(rename = "vBuy24h")]
    pub v_buy24h: Option<f64>,
    #[serde(rename = "vBuy24hUSD")]
    pub v_buy24h_usd: Option<f64>,
    #[serde(rename = "vBuyHistory24h")]
    pub v_buy_history24h: Option<f64>,
    #[serde(rename = "vBuyHistory24hUSD")]
    pub v_buy_history24h_usd: Option<f64>,
    #[serde(rename = "vBuy24hChangePercent")]
    pub v_buy24h_change_percent: Option<f64>,
    #[serde(rename = "vSell24h")]
    pub v_sell24h: Option<f64>,
    #[serde(rename = "vSell24hUSD")]
    pub v_sell24h_usd: Option<f64>,
    #[serde(rename = "vSellHistory24h")]
    pub v_sell_history24h: Option<f64>,
    #[serde(rename = "vSellHistory24hUSD")]
    pub v_sell_history24h_usd: Option<f64>,
    #[serde(rename = "vSell24hChangePercent")]
    pub v_sell24h_change_percent: Option<f64>,
    #[serde(rename = "numberMarkets")]
    pub number_markets: Option<i64>,
}

// A simplified version of market data for basic token info
#[derive(Debug, Deserialize, Default, Clone)]
pub struct TokenMarketData {
    pub address: String,
    pub price: f64,
    pub volume_24h: f64,
    pub decimals: u8,
    pub price_sol: f64,
    pub market_cap: f64,
    pub fully_diluted_market_cap: f64,
    pub circulating_supply: f64,
    pub total_supply: f64,
    pub price_change_24h: f64,
    pub volume_change_24h: f64,
}

impl From<TokenMarketResponse> for TokenMarketData {
    fn from(response: TokenMarketResponse) -> Self {
        Self {
            address: response.address,
            price: response.price.unwrap_or_default(),
            volume_24h: response.v24h.unwrap_or_default(),
            decimals: response.decimals as u8,
            price_sol: response.price.unwrap_or_default(), // Price is in SOL
            market_cap: response.market_cap.unwrap_or_default(),
            fully_diluted_market_cap: response.fdv.unwrap_or_default(),
            circulating_supply: response.circulating_supply.unwrap_or_default(),
            total_supply: response.total_supply.unwrap_or_default(),
            price_change_24h: response.price_change_24h_percent.unwrap_or_default(),
            volume_change_24h: response.v24h_change_percent.unwrap_or_default(),
        }
    }
}
