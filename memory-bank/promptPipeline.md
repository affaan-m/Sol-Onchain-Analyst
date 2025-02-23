# Token Filter Pipeline

## Current Status (2024-02-23)
- Pipeline fully functional with 5-stage filtering process
- Successfully integrates with BirdEye API and MongoDB
- Uses O1-MINI model for analysis (pending O3-MINI availability)

## Pipeline Stages
1. **BirdEye Filter Selection**
   - LLM selects 5 optimal filtering parameters
   - Hardcoded parameters: sort_by="liquidity", sort_type="desc", limit=100
   - Parameters chosen based on liquidity, market cap, trade activity, holders, and momentum

2. **Token List Retrieval**
   - Fetches token data from BirdEye API v3
   - Applies selected filters
   - Returns detailed token information including social/dev metrics

3. **Market Analysis**
   - Analyzes market metrics (liquidity, volume, momentum)
   - Scores tokens on multiple criteria
   - Filters out low-potential tokens

4. **Metadata Analysis**
   - Evaluates social signals and development metrics
   - Assesses risk factors
   - Provides detailed token analysis with strengths/risks

5. **Final Filtering & Storage**
   - Stores filtered tokens in MongoDB
   - Collection: "token_recommendations"
   - Includes comprehensive analysis and market context

## Data Structures
- **BirdeyeFilters**: API query parameters
- **TokenAnalysis**: Comprehensive token evaluation
- **FilterResponse**: Complete analysis results
- **FilterSummary**: Market overview and statistics

## LLM Integration
- Uses structured prompts for consistent analysis
- Returns standardized JSON responses
- Handles markdown code blocks and response cleaning

## MongoDB Storage
- Stores token recommendations with timestamps
- Includes detailed scoring and analysis
- Maintains market context for historical reference

## Next Steps
- Add visualization layer for analysis results
- Implement real-time monitoring capabilities
- Enhance social/dev metrics analysis