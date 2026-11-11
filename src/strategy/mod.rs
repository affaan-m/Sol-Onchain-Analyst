#[instrument(skip(self))]
pub async fn analyze_trading_opportunity(&self, prompt: String, sol_balance: f64) -> Result<String> {
    info!("Analyzing trading opportunity with prompt: {}", prompt);
    
    // Format the prompt with market analysis requirements
    let formatted_prompt = format!(
        "{}\n\nAnalyze this trading opportunity and provide a detailed recommendation in the following JSON format:\n{{
            \"action\": \"Buy|Sell|Hold\",
            \"token_address\": \"string\",
            \"amount_in_sol\": number,
            \"reasoning\": \"string\",
            \"confidence\": number (0.0-1.0),
            \"risk_assessment\": \"string\",
            \"market_analysis\": {{
                \"volume_analysis\": {{
                    \"current_volume_usd\": number,
                    \"volume_change_24h\": number,
                    \"is_volume_bullish\": boolean,
                    \"analysis\": \"string\"
                }},
                \"price_trend\": {{
                    \"current_trend\": \"string\",
                    \"support_levels\": [number],
                    \"resistance_levels\": [number],
                    \"trend_strength\": number (0.0-1.0)
                }},
                \"liquidity_assessment\": {{
                    \"liquidity_score\": number (0.0-1.0),
                    \"slippage_estimate\": number,
                    \"is_liquid_enough\": boolean
                }},
                \"momentum_indicators\": {{
                    \"rsi_14\": number,
                    \"macd\": {{
                        \"value\": number,
                        \"signal\": \"bullish|bearish|neutral\"
                    }},
                    \"overall_momentum\": \"strong_buy|buy|neutral|sell|strong_sell\"
                }},
                \"on_chain_metrics\": {{
                    \"unique_holders\": number,
                    \"holder_concentration\": number (0.0-1.0),
                    \"smart_money_flow\": \"inflow|outflow|neutral\"
                }}
            }},
            \"execution_strategy\": {{
                \"entry_type\": \"market|limit\",
                \"position_size_sol\": number,
                \"stop_loss_pct\": number,
                \"take_profit_levels\": [{{
                    \"price_target\": number,
                    \"size_pct\": number
                }}],
                \"time_horizon\": \"short|medium|long\",
                \"dca_strategy\": {{
                    \"should_dca\": boolean,
                    \"interval_hours\": number,
                    \"num_entries\": number
                }}
            }}
        }}\n\nAvailable SOL balance: {} SOL\n\nConsider the following criteria for the analysis:\n1. Volume should show significant increase (>50% 24h change) with sustainable growth\n2. Price action should show clear trend with identifiable support/resistance levels\n3. Liquidity should be sufficient to enter/exit position with <2% slippage\n4. Momentum indicators should align with the overall trend\n5. Smart money flow should indicate institutional interest\n6. Risk:reward ratio should be at least 1:3 for any trade", 
        prompt,
        sol_balance
    );

    // Get analysis from LLM
    let analysis = self.agent.complete(&formatted_prompt).await?;
    
    info!("Received analysis from LLM");
    Ok(analysis)
}