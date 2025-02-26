# Technical Context

## Core Technologies

### Backend

- Rust (2021 edition)
- Tokio async runtime
- MongoDB for data storage
- Birdeye API integration
- Claude 3.7 Sonnet integration (planned)

### Database

- MongoDB Atlas
- Collections:
  - `token_trending`
  - `token_analytics`
  - `token_recommendations`
  - `kol_wallets`
- Vector store support for embeddings
- Compound indexing for efficient queries

### AI & ML

- OpenAI API integration
  - Current: o1-mini model
  - Planned: o3-mini model
- Claude 3.7 Sonnet integration (planned)
  - Superior reasoning capabilities
  - Enhanced context window
  - Detailed thought process documentation
- Vector embeddings for similarity search
  - Dimension: 1536
  - Similarity metric: cosine

### APIs and Integration

- Birdeye API
  - Token trending data
  - Token analytics and market data
  - Rate limited with 500ms delay between requests
  - Response structure:
    - Basic token info (address, symbol, name, decimals)
    - Market metrics (price, volume, liquidity)
    - Supply information (total, circulating)
    - Extended metadata (fdv, holder metrics)
    - Trading data (last trade time, market count)
- Twitter API (planned)
  - Sentiment analysis
  - Community growth tracking
  - Engagement metrics

### Development Tools

- Cargo for build and dependency management
- Environment configuration via `.env`
- Tracing for logging and debugging
- Colored CLI output with indicatif

## Key Dependencies

### Core

```toml
anyhow = "1.0"
async-trait = "0.1"
bigdecimal = { version = "0.2", features = ["serde"] }
bson = "2.0"
mongodb = "3.2.1"
tokio = { version = "1", features = ["full"] }
```

### Blockchain

```toml
solana-sdk = "2.2.1"
solana-program = "2.2.1"
spl-token = "7.0"
```

### AI & ML

```toml
openai = "0.8.0"
claude-api-rs = "0.1.0"  # Planned
```

### Utilities

```toml
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
dotenvy = "0.15.7"
colored = "2.0"
indicatif = "0.17"
```

## Architecture Components

### Data Collection

1. Trending Token Capture
   - Fetches trending tokens from Birdeye
   - Stores in MongoDB with timestamps
   - Uses compound indexing for efficient queries

2. Token Analytics Processing
   - Processes trending tokens for detailed analytics
   - Calculates technical indicators
   - Stores comprehensive market data

### Token Filter Pipeline

1. BirdEye Filter Selection
   - Applies mandatory filters
   - Ensures minimum quality baseline
   - Logs filter parameters prominently

2. Token List Retrieval
   - Fetches and validates token data
   - Handles null and edge cases
   - Error recovery and logging

3. Multi-stage Analysis
   - Market metrics analysis
   - Social and development metrics evaluation
   - KOL ownership tracking
   - Detailed decision reasoning

### KOL Wallet Tracker

- Monitors influential trader wallets
- Documents token ownership by KOLs
- Updates token recommendations with ownership data
- Provides social proof validation

### MongoDB Vector Search

- 1536-dimensional embeddings
- Cosine similarity search
- Batch document insertion
- Connection pooling with error handling

## Cloud Deployment (Planned)

- Continuous operation on cloud infrastructure
- Scheduled analysis runs
- Centralized database access
- Alert system for high-potential tokens

## Development Setup

1. MongoDB Atlas cluster configuration
2. Environment variables in `.env`
3. Rust toolchain setup
4. Birdeye API key configuration
5. OpenAI API key configuration

## Technical Constraints

- Birdeye API rate limits
- MongoDB Atlas connection limits
- Memory usage for vector operations
- Network latency considerations
- Model context window limitations

## Advanced Analytics (Planned)

- Twitter sentiment analysis
- GitHub development activity metrics
- On-chain transaction pattern analysis
- Correlation with macro market trends
