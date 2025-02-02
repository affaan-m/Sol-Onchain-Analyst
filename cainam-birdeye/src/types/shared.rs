use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    Solana,
    Ethereum,
    Arbitrum,
    Avalanche,
    Bsc,
    Optimism,
    Polygon,
    Base,
    Zksync,
    Sui,
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Chain::Solana => write!(f, "solana"),
            Chain::Ethereum => write!(f, "ethereum"),
            Chain::Arbitrum => write!(f, "arbitrum"),
            Chain::Avalanche => write!(f, "avalanche"),
            Chain::Bsc => write!(f, "bsc"),
            Chain::Optimism => write!(f, "optimism"),
            Chain::Polygon => write!(f, "polygon"),
            Chain::Base => write!(f, "base"),
            Chain::Zksync => write!(f, "zksync"),
            Chain::Sui => write!(f, "sui"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub address: String,
    pub chain: Chain,
    #[serde(rename = "type")]
    pub address_type: AddressType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AddressType {
    Wallet,
    Token,
    Contract,
}

// Helper functions for address validation and chain detection
impl Address {
    pub fn detect_chain(address: &str) -> Chain {
        if address.starts_with("0x") {
            if address.len() == 66 {
                Chain::Sui // Sui addresses are 0x + 64 hex chars
            } else {
                Chain::Ethereum // Default to Ethereum for other 0x addresses
            }
        } else {
            Chain::Solana // Default to Solana for base58 addresses
        }
    }

    pub fn is_valid_address(address: &str) -> bool {
        match Self::detect_chain(address) {
            Chain::Solana => bs58::decode(address).into_vec().is_ok(),
            Chain::Sui => address.starts_with("0x") && address.len() == 66 && hex::decode(&address[2..]).is_ok(),
            _ => address.starts_with("0x") && address.len() == 42 && hex::decode(&address[2..]).is_ok(),
        }
    }
}

// Constants for API configuration
pub const API_BASE_URL: &str = "https://public-api.birdeye.so";
pub const DEFAULT_MAX_RETRIES: u32 = 3;
pub const RETRY_DELAY_MS: u64 = 2000; 