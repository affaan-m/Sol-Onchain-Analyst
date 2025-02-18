# Technical Context

## Core Technologies

### Backend
- Rust (2021 edition)
- Tokio async runtime
- MongoDB for data storage
- Birdeye API integration

### Database
- MongoDB Atlas
- Collections:
  - `trending_tokens`
  - `token_analytics`
  - Vector store support for embeddings
- Compound indexing for efficient queries

### APIs and Integration
- Birdeye API
  - Token trending data
  - Token analytics and market data
  - Rate limited with 500ms delay between requests

### Development Tools
- Cargo for build and dependency management
- Environment configuration via `.env`
- Tracing for logging and debugging

## Key Dependencies

### Core
```toml
anyhow = "1.0"
async-trait = "0.1"
bigdecimal = { version = "0.2", features = ["serde"] }
bson = "2.0"
mongodb = "3.2.1"
tokio = { version = "1", features = ["full"] }
```

### Blockchain
```toml
solana-sdk = "2.2.1"
solana-program = "2.2.1"
spl-token = "7.0"
```

### Utilities
```toml
tracing = "0.1"
serde = { version = "1.0", features = ["derive"] }
dotenvy = "0.15.7"
```

## Architecture Components

### Data Collection
1. Trending Token Capture
   - Fetches trending tokens from Birdeye
   - Stores in MongoDB with timestamps
   - Uses compound indexing for efficient queries

2. Token Analytics Processing
   - Processes trending tokens for detailed analytics
   - Calculates technical indicators
   - Stores comprehensive market data

### Services
- TokenAnalyticsService
- TokenDataService
- BirdeyeClient

## Development Setup
1. MongoDB Atlas cluster configuration
2. Environment variables in `.env`
3. Rust toolchain setup
4. Birdeye API key configuration

## Technical Constraints
- Birdeye API rate limits
- MongoDB Atlas connection limits
- Memory usage for vector operations
- Network latency considerations

## Vector Store Implementation

### MongoDB Atlas Setup

- Enabled Atlas Search for vector similarity search capabilities
- Created token_analytics collection with document structure for embeddings
- Implemented vector search index for efficient similarity search using cosine distance
- Added vector store integration with proper connection pooling

### Database Schema

The vector store implementation uses the following document structure:

```json
{
    "_id": ObjectId,
    "token_address": String,
    "token_name": String,
    "token_symbol": String,
    "embedding": Array<float>,
    "created_at": ISODate
}
```

### Search Configuration

Implemented MongoDB vector search with:

- Vector search index on embedding field
- Cosine similarity for distance calculation
- Configurable search parameters:
  - Exact matching option
  - Number of candidates
  - Field specification for embedding search

### Integration Notes

- Using OpenAI's text-embedding-3-small model (1536 dimensions)
- Configured with MongoDB Atlas Search for vector similarity
- Supports batch document insertion
- Includes proper connection pooling
- Implements retry logic for operations

### Current Implementation

1. MongoDB Connection Pool
   - Configurable min/max pool size
   - Connection timeout settings
   - Error handling for connection issues

2. Vector Store Operations
   - Document insertion with embeddings
   - Vector similarity search
   - Top-N query support
   - Proper error handling

3. Data Models
   - TokenAnalyticsData structure
   - Proper serialization/deserialization
   - ObjectId handling
   - Embedding field management

### Error Handling

- Comprehensive error types for MongoDB operations
- Connection error handling
- Vector store operation error handling
- Proper error propagation
- Logging integration with tracing

### Pending Improvements

1. SearchParams configuration refinement
2. Enhanced error context for vector operations
3. Additional logging for debugging
4. Performance optimization for batch operations
5. Connection pool monitoring
