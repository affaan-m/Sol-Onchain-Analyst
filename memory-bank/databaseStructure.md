# MongoDB Database Structure

## Database: `cainam`

### Market Data Collections

#### 1. `trending_tokens`

**Purpose:** Stores trending token data from Birdeye API
**API Endpoint:** `https://public-api.birdeye.so/defi/token_trending`
**Update Frequency:** Every 5 minutes
**Schema and Field Types:**

```json
{
    "_id": { "type": "objectId" },
    "address": { "type": "string", "searchable": true },
    "decimals": { "type": "number" },
    "liquidity": { "type": "sortableNumberBetaV1" },
    "logo_uri": { "type": "string" },
    "name": { "type": "string", "searchable": true },
    "symbol": { "type": "token" },
    "volume_24h_usd": { "type": "sortableNumberBetaV1" },
    "volume_24h_change_percent": { "type": "number" },
    "fdv": { "type": "sortableNumberBetaV1" },
    "marketcap": { "type": "sortableNumberBetaV1" },
    "rank": { "type": "numberFacet" },
    "price": { "type": "sortableNumberBetaV1" },
    "price_24h_change_percent": { "type": "number" },
    "timestamp": { "type": "sortableDateBetaV1" }
}
```

**Indexes:**

- Compound: `{ address: 1, timestamp: -1 }` (Primary for querying historical data)
- Single: `{ timestamp: -1 }` (For recent trending queries)

#### 2. `token_analytics`

**Purpose:** Stores detailed token analytics and metrics
**API Endpoint:** `https://public-api.birdeye.so/defi/token_overview?address={token_address}`
**Update Frequency:** Every 15 minutes for active tokens
**Schema and Field Types:**

```json
{
    "_id": { "type": "objectId" },
    "token_address": { "type": "string", "searchable": true },
    "token_name": { "type": "string", "searchable": true },
    "token_symbol": { "type": "token" },
    "decimals": { "type": "number" },
    "logo_uri": { "type": "string" },
    
    // Price metrics
    "price": { "type": "sortableNumberBetaV1" },
    "price_change_24h": { "type": "number" },
    "price_change_7d": { "type": "number" },
    
    // Volume metrics
    "volume_24h": { "type": "sortableNumberBetaV1" },
    "volume_change_24h": { "type": "number" },
    "volume_by_price_24h": { "type": "sortableNumberBetaV1" },
    
    // Market metrics
    "market_cap": { "type": "sortableNumberBetaV1" },
    "fully_diluted_market_cap": { "type": "sortableNumberBetaV1" },
    "circulating_supply": { "type": "sortableNumberBetaV1" },
    "total_supply": { "type": "sortableNumberBetaV1" },
    
    // Liquidity metrics
    "liquidity": { "type": "sortableNumberBetaV1" },
    "liquidity_change_24h": { "type": "number" },
    
    // Trading metrics
    "trades_24h": { "type": "numberFacet" },
    "average_trade_size": { "type": "sortableNumberBetaV1" },
    
    // Holder metrics
    "holder_count": { "type": "numberFacet" },
    "active_wallets_24h": { "type": "numberFacet" },
    "whale_transactions_24h": { "type": "numberFacet" },
    
    // Technical indicators
    "rsi_14": { "type": "sortableNumberBetaV1" },
    "macd": { "type": "sortableNumberBetaV1" },
    "macd_signal": { "type": "sortableNumberBetaV1" },
    "bollinger_upper": { "type": "sortableNumberBetaV1" },
    "bollinger_lower": { "type": "sortableNumberBetaV1" },
    
    // Social metrics
    "social_score": { "type": "sortableNumberBetaV1" },
    "social_volume": { "type": "numberFacet" },
    "social_sentiment": { "type": "sortableNumberBetaV1" },
    "dev_activity": { "type": "numberFacet" },
    
    // Timestamps
    "timestamp": { "type": "sortableDateBetaV1" },
    "created_at": { "type": "date" },
    "last_trade_time": { "type": "date" },
    
    // Metadata and extensions
    "metadata": { "type": "document" },
    "embedding": { "type": "knnVector", "dimensions": 1536, "similarity": "cosine" }
}
```

**Indexes:**

- Compound: `{ token_address: 1, timestamp: -1 }` (Primary for token history)
- Compound: `{ timestamp: -1, volume_24h: -1 }` (For high volume analysis)
- Vector: `{ embedding: "vector", dimensions: 1536 }` (For similarity search)

### Future Collections (To Be Implemented)

#### 3. `market_signals`

**Purpose:** Store trading signals generated from analytics
**Schema and Field Types:**

```json
{
    "_id": { "type": "objectId" },
    "token_address": { "type": "string", "searchable": true },
    "signal_type": { "type": "stringFacet" },
    "confidence": { "type": "sortableNumberBetaV1" },
    "risk_score": { "type": "sortableNumberBetaV1" },
    "price": { "type": "sortableNumberBetaV1" },
    "volume_change": { "type": "sortableNumberBetaV1" },
    "timestamp": { "type": "sortableDateBetaV1" },
    "metadata": { "type": "document" }
}
```

#### 4. `trading_positions`

**Purpose:** Track active and historical trading positions
**Schema and Field Types:**

```json
{
    "_id": { "type": "objectId" },
    "token_address": { "type": "string", "searchable": true },
    "entry_price": { "type": "sortableNumberBetaV1" },
    "current_price": { "type": "sortableNumberBetaV1" },
    "position_size": { "type": "sortableNumberBetaV1" },
    "position_type": { "type": "stringFacet" },
    "entry_time": { "type": "sortableDateBetaV1" },
    "last_update": { "type": "date" },
    "pnl": { "type": "sortableNumberBetaV1" },
    "status": { "type": "stringFacet" }
}
```

## Field Type Usage Guidelines

1. **sortableNumberBetaV1**: Used for numeric fields that need sorting and range queries
   - Prices, volumes, market caps, liquidity values
   - Technical indicators
   - Performance metrics

2. **numberFacet**: Used for numeric fields that need faceting/aggregation
   - Counts (trades, holders, transactions)
   - Ranks
   - Activity metrics

3. **sortableDateBetaV1**: Used for date fields that need sorting and range queries
   - Primary timestamps
   - Entry times
   - Update times

4. **date**: Used for simple date fields without sorting requirements
   - Creation dates
   - Last update timestamps

5. **stringFacet**: Used for categorical string fields
   - Status values
   - Signal types
   - Position types

6. **token**: Used for token symbols and other tokenized text
   - Cryptocurrency symbols
   - Standardized identifiers

7. **string**: Used for general text fields
   - Names
   - Addresses
   - URLs

8. **document**: Used for nested/complex data
   - Metadata
   - Extended properties

9. **knnVector**: Used for vector similarity search
   - Embeddings for semantic search
   - Feature vectors

## Relationships

- `trending_tokens` → `token_analytics`: Trending tokens trigger detailed analytics collection
- `token_analytics` → `market_signals`: Analytics data generates trading signals
- `market_signals` → `trading_positions`: Signals may lead to new trading positions

## Notes

1. All monetary values use `sortableNumberBetaV1` for efficient sorting and range queries
2. Timestamps use `sortableDateBetaV1` for time-series operations
3. Categorical fields use `stringFacet` or `numberFacet` for aggregations
4. Search-critical fields are marked as `searchable: true`
5. Vector search uses `knnVector` with cosine similarity

## API Rate Limits

- Birdeye API: 500ms minimum delay between requests
- Recommended batch size: 100 tokens per analytics update
- Keep-alive connections for MongoDB

## Monitoring Considerations

1. Collection sizes and growth rates
2. Index usage and performance
3. Query patterns and optimization
4. Backup strategy and retention policy
