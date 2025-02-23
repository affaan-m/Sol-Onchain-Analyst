# Cainam Core

A decentralized AI trading agent platform on Solana, leveraging RIG for LLM interactions and MongoDB for vector storage.

## Token Filter Pipeline

The token filter pipeline is a core component that analyzes and filters Solana tokens using the BirdEye API and LLM-based analysis. The pipeline consists of five stages:

1. **BirdEye Filter Selection**: Intelligent selection of filtering parameters using LLM
2. **Token List Retrieval**: Fetching and filtering tokens from BirdEye API
3. **Market Analysis**: Deep analysis of market metrics and trading patterns
4. **Metadata Analysis**: Evaluation of social signals and development metrics
5. **Final Filtering & Storage**: Storage of filtered tokens with comprehensive analysis

### Features

- Fully automated token analysis and filtering
- Integration with BirdEye API for real-time market data
- MongoDB storage for token recommendations
- LLM-powered market analysis and risk assessment
- Comprehensive scoring system for token evaluation

### Technical Stack

- **Language**: Rust
- **APIs**: BirdEye API (Solana)
- **Database**: MongoDB
- **LLM Integration**: RIG with O1-MINI model
- **Data Storage**: Vector storage for token analysis

### Getting Started

1. Clone the repository
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Set up environment variables:
   ```bash
   BIRDEYE_API_KEY=your_api_key
   OPENAI_API_KEY=your_api_key
   MONGODB_URI=your_mongodb_uri
   ```
4. Run the token filter example:
   ```bash
   cargo run --example token_filter
   ```

### Pipeline Configuration

The token filter pipeline can be configured through:
- BirdEye API parameters in `src/prompts/token_filter_initial.txt`
- MongoDB collection settings in `src/services/token_filter.rs`
- LLM model selection in environment variables

### Data Structures

- **BirdeyeFilters**: API query parameters
- **TokenAnalysis**: Token evaluation data
- **FilterResponse**: Analysis results
- **FilterSummary**: Market statistics

### Next Steps

- [ ] Add visualization layer for analysis results
- [ ] Implement real-time monitoring
- [ ] Enhance social/dev metrics analysis
- [ ] Create web dashboard for token recommendations

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a new Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Overview

Cainam Core is a Rust-based system that implements autonomous AI trading agents, market monitoring, and data analysis for the Solana blockchain. The system features real-time market data processing, automated trading execution, and advanced risk management capabilities.

### Key Features

- Real-time market monitoring via Birdeye API
- Blockchain transaction monitoring using Helius webhooks
- Autonomous trading agents with AI-driven decision making
- Advanced risk management and position sizing
- Time-series data storage with TimescaleDB
- Vector similarity search using Qdrant
- Discord and Twitter integration

## Prerequisites

- Rust 1.75+ (2021 edition)
- PostgreSQL 15+ with TimescaleDB extension
- Solana CLI tools
- Node.js and npm (for development tools)

## Installation

1. Clone the repository:

```bash
git clone https://github.com/cainamventures/cainam-core
cd cainam-core
```

2. Copy the environment template and configure your variables:

```bash
cp .env.example .env
# Edit .env with your configuration
```

3. Install development dependencies:

```bash
# Install pre-commit hooks
pre-commit install

# Install required database extensions
psql -c 'CREATE EXTENSION IF NOT EXISTS timescaledb;'
```

4. Build the project:

```bash
cargo build
```

## Configuration

The following environment variables are required:

```env
# Database
DATABASE_URL=postgresql://user:password@localhost/dbname

# Solana
SOLANA_RPC_URL=your_rpc_url
HELIUS_API_KEY=your_helius_key

# APIs
BIRDEYE_API_KEY=your_birdeye_key

# Optional integrations
DISCORD_TOKEN=your_discord_token
TWITTER_API_KEY=your_twitter_key
```

## Project Structure

```
src/
├── actions/      # External API interactions
├── agent/        # Agent implementations
├── trading/      # Trading logic
├── models/       # Data models
└── services/     # Business logic
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suite
cargo test --package cainam-core
```

### Database Migrations

```bash
# Apply migrations
sqlx migrate run

# Create new migration
sqlx migrate add <name>
```

### Code Style

The project uses rustfmt and clippy for code formatting and linting:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy
```

## Performance Requirements

- Trade execution: < 500ms end-to-end
- Market data updates: < 1s refresh rate
- Signal processing: < 200ms
- Database queries: < 100ms response time

## Dependencies

Core dependencies include:

- tokio (async runtime)
- solana-client & solana-sdk (blockchain interaction)
- serde (serialization)
- tokio-postgres (database)
- qdrant-client (vector store)
- rig-core (framework)

## Contact

- Author: Matt Gunnin
- Email: <matt@cainamventures.com>
- Repository: <https://github.com/cainamventures/cainam-core>

## CLI Usage

The `cainam` CLI tool provides several commands for interacting with the platform:

### Get Trending Tokens

```bash
cargo run --bin cainam trending
```

This command displays a list of trending tokens with their current prices, 24h changes, volumes, and market caps.

### Get Token Overview

```bash
cargo run --bin cainam token <TOKEN_ADDRESS>
```

Shows detailed information about a specific token, including:
- Current price and market cap
- 24h volume and price change
- Number of holders and active wallets

### Analyze Market Signals

```bash
cargo run --bin cainam signals <TOKEN_ADDRESS>
```

Analyzes market signals for a specific token, displaying:
- Signal type (if any)
- Confidence score
- Risk assessment
- Price and volume changes

### Monitor Tokens

```bash
cargo run --bin cainam monitor <TOKEN_ADDRESS1,TOKEN_ADDRESS2,...> [--interval <SECONDS>]
```

Continuously monitors specified tokens for market signals. The interval defaults to 300 seconds (5 minutes).

### CLI Usage

The token filter pipeline can be run through a user-friendly CLI interface that provides real-time feedback and colored output:

```bash
# Run the token filter with continuous monitoring
cargo run --example token_filter_cli filter --continuous

# Run a single token filter pass
cargo run --example token_filter_cli filter
```

The CLI provides:
- Colored section headers and progress indicators
- Real-time token analysis with detailed metrics
- Visual score bars for different metrics
- Color-coded market signals and risk assessments

Example output includes:
- Token information (name, symbol, price) in bold white
- Prices in green
- Market cap in yellow
- Volume in blue
- Price changes (positive in green, negative in red)
- Score bars (green ≥0.8, yellow ≥0.6, red <0.6)
- Progress spinners for long-running operations

### CLI Features

- **Progress Tracking**: Real-time progress indicators for long-running operations
- **Color-Coded Output**: Intuitive color scheme for different metrics
- **Visual Score Bars**: Progress bar style visualization of scores
- **Section Headers**: Clear organization of output with colored headers
- **Market Signals**: Detailed market signal information with confidence scores
- **Analysis Summary**: Overview of token analysis with pass/fail counts

## BirdEye V3 API Integration Testing

The `feature/birdeye-v3-llm-filter` branch implements enhanced token filtering using BirdEye V3 API and LLM-based analysis. To test the implementation:

1. Clone and Setup:
   ```bash
   git clone https://github.com/CainamVentures/cainam-core.git
   cd cainam-core
   git checkout feature/birdeye-v3-llm-filter
   ```

2. Environment Setup:
   Create a `.env` file with:
   ```
   BIRDEYE_API_KEY=<your-key>
   OPENAI_API_KEY=<your-key>
   MONGODB_URI=<your-uri>
   MONGODB_DATABASE=<your-db>
   ```

3. Build and Run:
   ```bash
   cargo build
   cargo run --example token_filter
   ```

4. Verify Operation:
   - Check logs for successful API calls
   - Monitor MongoDB for stored analysis
   - Verify token filtering results

The branch is ready to merge into main. All tests are passing, and the implementation is production-ready.
