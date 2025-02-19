# MongoDB Database Structure for CAINAM (Revised)

## Database: `cainam`

This document outlines the collections within the `cainam` database, their purposes, schemas, indexes, and relationships. It also notes the relevant API endpoints for data ingestion.

### 1. Market Data Collections

#### 1.1. `trending_tokens`

* **Purpose:** Stores trending token data from the Birdeye API. This is the initial point of discovery for potentially interesting tokens.
* **API Endpoint:** `https://public-api.birdeye.so/defi/token_trending`
* **Update Frequency:** Every 5 minutes.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "address": string,          // Token contract address
        "decimals": number,         // Token decimals
        "liquidity": number,        // Current liquidity in USD
        "logo_uri": string,         // Token logo URL
        "name": string,             // Token name
        "symbol": string,           // Token symbol
        "volume_24h_usd": number,   // 24h volume in USD
        "volume_24h_change_percent": number,  // 24h volume change %
        "fdv": number,              // Fully diluted valuation
        "marketcap": number,        // Current market cap
        "rank": number,             // Trending rank
        "price": number,            // Current price
        "price_24h_change_percent": number,   // 24h price change %
        "timestamp": date       // When this data was captured
    }
    ```

* **Indexes:**
  * Compound: `{ address: 1, timestamp: -1 }` (For querying historical data for a specific token)
  * Single: `{ timestamp: -1 }` (For querying the most recent trending tokens)
* **Relationship:** Feeds into `token_analytics`.

#### 1.2. `token_analytics`

* **Purpose:** Stores comprehensive token analytics, combining data from Birdeye with calculated metrics and AI-generated insights.
* **API Endpoints:**
  * `https://public-api.birdeye.so/defi/token_overview?address={token_address}` (Token Overview)
  * `https://public-api.birdeye.so/defi/v3/token/market-data?address={token_address}` (Market Data)
  * `https://public-api.birdeye.so/defi/v3/token/trade-data/multiple?list_address={address1},{address2},...` (Trade Data - for multiple tokens, batched)
* **Update Frequency:** Every 15 minutes for actively tracked tokens.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "token_address": string,
        "symbol": string,
        "name": string,
        "decimals": number,
        "logo_uri": string,

        //-- Price Data (from Birdeye and calculated)
        "price": number,
        "price_change_24h": number,
        "price_change_7d": number,  // Calculated from historical data
        "price_high_24h": number,    // From OHLCV data or calculated
        "price_low_24h": number,     // From OHLCV data or calculated

        //-- Volume Data (from Birdeye)
        "volume_24h": number,
        "volume_change_24h": number,
        "volume_by_price_24h": document, // Could be a nested document with price ranges and volumes

        //-- Market Cap and Supply (from Birdeye)
        "market_cap": number,
        "fully_diluted_market_cap": number,
        "circulating_supply": number,
        "total_supply": number,

        //-- Liquidity (from Birdeye)
        "liquidity": number,
        "liquidity_change_24h": number, // Calculated

        //-- Trading Metrics (from Birdeye and calculated)
        "trades_24h": number,
        "average_trade_size": number, // Calculated (volume_24h / trades_24h)
        "buy_volume_24h": number,      // From Birdeye trade data
        "sell_volume_24h": number,     // From Birdeye trade data

        //-- Holder Metrics (from Birdeye, potentially supplemented by other sources)
        "holder_count": number,
        "active_wallets_24h": number, // Requires additional on-chain analysis
        "whale_transactions_24h": number, // Requires additional on-chain analysis, defining "whale" threshold

        //-- Technical Indicators (Calculated)
        "rsi_14": number,
        "sma_20": number,
        "ema_50": number,
        "bollinger_bands": document, // { upper: number, middle: number, lower: number }
        "macd": document,          // { macd: number, signal: number, histogram: number }

        //-- Sentiment Analysis (from Analyst Agent)
        "social_sentiment": document, // { overall_score: number, twitter_score: number, telegram_score: number, ... }
        "news_sentiment": number,

        //-- On-Chain Analysis (from Analyst Agent, potentially using Birdeye's wallet APIs)
        "whale_activity_index": number, // (0-1, based on whale transaction volume and count)
        "network_growth": number,      // New addresses interacting with the token
        "concentration_ratio": number,  // Percentage of supply held by top N addresses

        //-- Risk Metrics (from Risk Manager Agent)
        "volatility_30d": number,     // Annualized volatility
        "value_at_risk_95": number,  // 95% VaR
        "expected_shortfall_95": number, // 95% Expected Shortfall

        "timestamp": date
    }
    ```

* **Indexes:**
  * Compound: `{ token_address: 1, timestamp: -1 }`
  * Single: `{ timestamp: -1 }`
  * Single: `{ "social_sentiment.overall_score": 1 }`
  * Single: `{ "whale_activity_index": 1 }`
  * Single: `{ "rsi_14": 1 }`
* **Relationship:** Fed by `trending_tokens` and Birdeye API calls. Generates data for `market_signals`.

### 2. Trading and Strategy Collections

#### 2.1. `market_signals`

* **Purpose:** Stores trading signals generated by the Analyst Agent.
* **API Endpoint:** None (Internally generated).
* **Update Frequency:** Continuously.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "asset_address": string,
        "signal_type": string, // "PriceSpike", "VolumeSurge", "SentimentChange", "WhaleActivity", "TechnicalIndicatorCrossover", "NewsEvent"
        "direction": string,   // "BUY", "SELL", "HOLD"
        "confidence": number,
        "risk_score": number,
        "price": number,
        "timestamp": date,
        "metadata": document  // e.g., { rsi_value: 75, moving_average_crossover: "20_50_bullish", news_headline: "...", news_url: "..." }
    }
    ```

* **Indexes:**
  * Compound: `{ asset_address: 1, timestamp: -1 }`
  * Single: `{ timestamp: -1 }`
  * Single: `{ signal_type: 1 }`
  * Single: `{ confidence: 1 }`
* **Relationship:** Fed by `token_analytics`. May trigger actions in `trading_positions`.

#### 2.2. `trading_positions`

* **Purpose:** Tracks active and historical trading positions.
* **API Endpoint:** None (Internally generated).
* **Update Frequency:**  Real-time.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "token_address": string,
        "entry_price": number,
        "current_price": number,
        "position_size": number, // Quantity of tokens
        "position_type": string, // "LONG", "SHORT"
        "entry_time": date,
        "last_update": date,
        "pnl": number,
        "status": string, // "ACTIVE", "CLOSED"
        "exit_price": number,
        "exit_time": date,
        "stop_loss": number,      // Stop-loss price (if set)
        "take_profit": number,    // Take-profit price (if set)
        "leverage": number,       // Leverage used (if applicable)
        "liquidation_price": number // Liquidation price (if applicable)
    }
    ```

* **Indexes:**
  * Compound: `{ token_address: 1, entry_time: -1 }`
  * Single: `{ status: 1 }`
  * Single: `{ last_update: -1 }`
* **Relationship:** Triggered by `market_signals`.

#### 2.3. `trade_history`

* **Purpose:** Records *all* executed trades (audit trail).
* **API Endpoint:** None (Internally generated).
* **Update Frequency:** Real-time.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "trader_address": string,
        "token_address": string,
        "trade_type": string,  // "BUY", "SELL"
        "quantity": number,
        "price": number,
        "timestamp": date,
        "status": string, // "FILLED", "PARTIALLY_FILLED", "CANCELLED", "REJECTED"
        "transaction_hash": string,
        "slippage": number,
        "fees": number,
        "order_type": string, // "MARKET", "LIMIT", "STOP_LOSS", "TAKE_PROFIT"
        "dex": string         // e.g., "Orca", "Raydium", "Jupiter"
    }
    ```

* **Indexes:**
  * Compound: `{ trader_address: 1, timestamp: -1 }`
  * Single: `{ token_address: 1, timestamp: -1 }`
  * Single: `{ status: 1 }`
* **Relationship:** Created by Trader Agent.

### 3. Risk Management and Portfolio Collections

#### 3.1. `risk_models`

* **Purpose:** Stores risk model parameters and outputs.
* **API Endpoint:** None (Internally generated).
* **Update Frequency:** Periodic.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "model_type": string, // "VaR", "ExpectedShortfall", "Volatility", "CorrelationMatrix"
        "asset_address": string, // "ALL" for portfolio-level, or a specific token address
        "parameters": document, // { confidence_level: 0.95, lookback_period: 30, ... }
        "output": number,      // e.g., VaR value, volatility, correlation coefficient
        "timestamp": date
    }
    ```

* **Indexes:**
  * Compound: `{ model_type: 1, asset_address: 1, timestamp: -1 }`
  * Single: `{ timestamp: -1 }`

#### 3.2. `portfolio_allocations`

* **Purpose:** Stores target and actual portfolio allocations.
* **API Endpoint:** None (Internally generated).
* **Update Frequency:**  Whenever rebalancing occurs.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "wallet_address": string,
        "token_address": string,
        "target_allocation": number, // Percentage
        "actual_allocation": number,
        "timestamp": date
    }
    ```

* **Indexes:**
  * Compound: `{ wallet_address: 1, token_address: 1, timestamp: -1 }`
  * Single: `{ timestamp: -1 }`

### 4. Vector Embeddings

#### 4.1. `vectors`

* **Purpose:** Stores vector embeddings for similarity search.
* **API Endpoint:** None (Internally generated).
* **Update Frequency:** As needed.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "entity_type": string, // "token", "news_article", "tweet", "trading_strategy"
        "entity_id": string,   // Token address, URL, strategy ID, etc.
        "vector": [number],    // Array of numbers (embedding)
        "metadata": document,  // { timestamp: date, source: string, ... }
         "weights": {
              "vector": number,
              "metadata.timestamp": number
          },
          "name": string,
          "background": boolean
    }
    ```

* **Indexes:**
  * `{ "vector": "2dsphere", "metadata.timestamp": -1 }` (For geospatial and time-based similarity search)
  * `"weights": { "vector": 1, "metadata.timestamp": 1 }, "name": "vector_search_idx", "background": true`

### 5. Compliance Data (Optional)

#### 5.1. `compliance_records`

* **Purpose:** Stores compliance check records.
* **API Endpoint:** None (Internally generated).
* **Update Frequency:** Real-time.
* **Schema:**

    ```json
    {
        "_id": objectId,
        "transaction_hash": string,
        "check_type": string, // "KYC", "AML", "SanctionsList", "Jurisdiction"
        "result": string,    // "PASS", "FAIL", "PENDING"
        "timestamp": date,
        "details": document   // { reason: "...", flagged_address: "...", ... }
    }
    ```

* **Indexes:**
  * `{ transaction_hash: 1, timestamp: -1 }`
  * `{ check_type: 1, result: 1 }`
