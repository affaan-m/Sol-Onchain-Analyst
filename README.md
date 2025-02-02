# Cainam Core

Core functionality for the Cainam project - A decentralized network of autonomous AI trading agents for the $CAINAM token platform on Solana.

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

## Contributing

Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on contributing to the project.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contact

- Author: Matt Gunnin
- Email: <matt@cainamventures.com>
- Repository: <https://github.com/cainamventures/cainam-core>
