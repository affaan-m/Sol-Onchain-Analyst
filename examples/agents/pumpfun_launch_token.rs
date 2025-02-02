

use solagent::{Config, SolanaAgentKit};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let config = Config { openai_api_key: Some("your_api_key".to_string()), ..Default::default() };
    let agent = Arc::new(SolanaAgentKit::new("private_key", "RPC_URL", config));
    let res = agent
        .launch_token_pumpfun(
            "Name",
            "Symbol",
            "this is a description.",
            "https://www.baidu.com/img/PCtm_d9c8750bed0b3c7d089fa7d55720d6cf.png",
            None,
        )
        .await
        .unwrap();

    println!("Pumpfun Token response: {:?}", res);
}
