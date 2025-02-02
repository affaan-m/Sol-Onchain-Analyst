

use crate::RUGCHECK_URL;
use serde_json::Value;

/// Fetches a detailed report for a specific token.
///
/// # Parameters
///
/// - `mint` - The mint address of the token.
///
/// # Returns
/// Token detailed report.
///
/// # Errors
/// Throws an error if the API call fails.
pub async fn fetch_detailed_report(mint: String) -> Result<Value, Box<dyn std::error::Error>> {
    let url = format!("{}/tokens/{}/report", RUGCHECK_URL, mint);

    let response = reqwest::get(&url).await?;
    if !response.status().is_success() {
        return Err(format!("HTTP error! status: {}", response.status()).into());
    }

    let data: serde_json::Value = response.json().await?;
    Ok(data)
}
