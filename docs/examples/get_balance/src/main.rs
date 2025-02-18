// Copyright 2025 zTgx
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use solagent_core::{Config, SolanaAgentKit};
use solagent_plugin_solana::{get_balance, get_wallet_address};

#[tokio::main]
async fn main() {
    // Load configuration from environment variables
    let config = Config::from_env();
    let agent = SolanaAgentKit::new_from_env(config);

    // Get and display wallet address
    let wallet_address = get_wallet_address(&agent);
    println!("Wallet address: {}", wallet_address);

    match get_balance(&agent, None).await {
        Ok(balance) => println!("Account balance: {} SOL", balance),
        Err(e) => eprintln!("Error getting balance: {}", e),
    }
}
