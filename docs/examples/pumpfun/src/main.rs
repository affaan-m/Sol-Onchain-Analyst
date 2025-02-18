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

use solagent_core::{solana_sdk::signature::Keypair, Config, SolanaAgentKit};
use solagent_plugin_pumpfun::launch_token_pumpfun;

#[tokio::main]
async fn main() {
    // Create a new keypair
    let keypair = Keypair::new();
    // Encode the secret key to base58
    let private_key = keypair.to_base58_string();

    let config = Config { cookie_api_key: Some("".to_string()), ..Default::default() };
    let agent = SolanaAgentKit::new(&private_key, "https://api.devnet.solana.com", config);

    let res = launch_token_pumpfun(
        &agent,
        "Matt",
        "$MATT",
        "This is a test token for Matt.",
        "https://pbs.twimg.com/profile_images/1708966909952073729/XrWDSfm4_400x400.jpg",
        None,
    )
    .await
    .unwrap();

    println!("Pumpfun Token response: {:?}", res);
}
