# Project Brief

## Project Overview

Cainam Core is a Rust-based autonomous trading system for the Solana blockchain, focusing on market data collection, analysis, and automated trading execution.

## Core Requirements

### 1. Market Data Collection

- Capture trending tokens from Birdeye API
- Collect detailed token analytics
- Store historical market data
- Implement efficient data indexing

### 2. Market Analysis

- Calculate technical indicators
- Generate trading signals
- Assess market conditions
- Evaluate trading opportunities

### 3. Enhanced Token Filtering

- Multi-stage LLM-based filtration process
- 5 mandatory base quality filters
- KOL wallet ownership tracking
- Detailed reasoning and investment thesis generation
- Vector-based similarity search
- Real-time monitoring of emerging opportunities

### 4. Trading Automation

- Execute trades based on signals
- Manage portfolio positions
- Implement risk management
- Track trading performance

## Technical Goals

- Reliable data collection pipeline
- Efficient MongoDB integration with vector search
- Advanced LLM-powered token analysis
- Scalable architecture
- Robust error handling
- Comprehensive logging
- Performance optimization

## Product Features

### Token Filter Pipeline

The token filter pipeline provides a sophisticated multi-stage approach to identifying high-potential Solana tokens:

1. **Initial Filtration**
   - Applies 5 mandatory quality filters
   - Ensures minimum liquidity, market cap, holder count
   - Validates trading activity metrics
   - Eliminates low-quality tokens early

2. **Advanced Analysis**
   - Market metrics evaluation
   - Social signal verification
   - Development activity assessment
   - Risk factor identification
   
3. **KOL Wallet Tracking**
   - Monitors influential trader wallets
   - Documents token ownership by KOLs
   - Provides social proof validation
   - Increases confidence in promising tokens

4. **Decision Reasoning**
   - Comprehensive investment thesis generation
   - Detailed market analysis
   - Sentiment and social signal evaluation
   - Risk assessment and recommendations

5. **CLI Visualization**
   - Color-coded metric display
   - Visual presentation of analysis
   - KOL ownership information
   - Detailed reasoning output

### Planned Enhancements

1. **Twitter API Integration**
   - Enhanced sentiment analysis
   - Real-time social monitoring
   - Community growth tracking

2. **Claude 3.7 Reasoning**
   - Superior analysis quality
   - Detailed thought process documentation
   - Enhanced decision explanation

3. **Cloud-Based Continuous Monitoring**
   - 24/7 token scanning
   - Alert system for opportunities
   - Historical trend analysis

## Project Scope

- Market data collection and storage
- Technical analysis implementation
- Trading signal generation
- Automated trade execution
- Performance monitoring
- Risk management system

## Success Criteria

- Accurate market data capture
- Reliable signal generation
- High-quality token recommendations
- Comprehensive decision reasoning
- Efficient trade execution
- Scalable data storage
- Robust error handling
- Comprehensive monitoring
