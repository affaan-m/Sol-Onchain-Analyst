

/// Fetches the price of a given token quoted in USDC using Jupiter API.
///
/// # Parameters
///
/// - `mint`: The token mint address as a string.
///
/// # Returns
///
/// The token data.
pub async fn get_token_data_by_address(mint: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let url = format!("https://tokens.jup.ag/token/{}", mint);
    let response = reqwest::get(&url).await?;
    if !response.status().is_success() {
        return Err(format!("Failed to get token data: {}", response.status()).into());
    }

    let data: serde_json::Value = response.json().await?;
    Ok(data)
}
