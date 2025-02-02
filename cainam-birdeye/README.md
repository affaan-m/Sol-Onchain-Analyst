# cainam-birdeye

A Birdeye plugin for rig-core that provides token and wallet analytics on Solana.

## Features

- Token search with sorting and filtering options
- Wallet portfolio analysis
- Token price history and market data
- Token security analysis
- Support for multiple time intervals

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
cainam-birdeye = "0.1.0"
```

## Configuration

The plugin requires a Birdeye API key. You can obtain one from [Birdeye](https://birdeye.so).

Set the API key in your environment:

```bash
export BIRDEYE_API_KEY=your_api_key_here
```

Or provide it through your rig-core configuration.

## Usage

### Token Search

```rust
use rig_birdeye::{TokenSearchAction, TokenSortBy, SortType};

let action = TokenSearchAction {
    keyword: "SOL".to_string(),
    sort_by: Some(TokenSortBy::Volume24h),
    sort_type: Some(SortType::Desc),
    limit: Some(10),
};

let result = agent.execute(action).await?;
```

### Wallet Portfolio

```rust
use rig_birdeye::WalletSearchAction;

let action = WalletSearchAction {
    wallet: "wallet_address_here".to_string(),
};

let portfolio = agent.execute(action).await?;
```

## Actions

- `TokenSearchAction`: Search for tokens with various sorting options
- `WalletSearchAction`: Get wallet portfolio information

## Types

### TokenMarketData

```rust
pub struct TokenMarketData {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub price: f64,
    pub price_change_24h: f64,
    pub volume_24h: f64,
    pub market_cap: Option<f64>,
}
```

### WalletPortfolio

```rust
pub struct WalletPortfolio {
    pub wallet: String,
    pub total_usd: f64,
    pub items: Vec<WalletToken>,
}
```

## Error Handling

The plugin uses custom error types that implement `std::error::Error`:

```rust
pub enum BirdeyeError {
    RequestError(reqwest::Error),
    ApiError { status_code: u16, message: String },
    RateLimitExceeded,
    InvalidApiKey,
    // ...
}
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 