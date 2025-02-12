# Code Review Guidelines

Last Updated: 2025-02-11

## Focus Areas

### 1. MongoDB Integration

- Connection pooling configuration
- Error handling and retry logic
- Proper use of MongoDB Atlas features
- Vector store implementation

### 2. Vector Search Implementation

- Proper embedding handling
- Search parameter configuration
- Index creation and management
- Query optimization

### 3. Error Handling

```rust
// Good: Proper error context and handling
pub async fn search_tokens(query: &str) -> Result<Vec<TokenAnalytics>> {
    let results = pool.top_n("token_analytics", model, query, 10)
        .await
        .context("Failed to perform vector search")?;
    
    process_results(results)
        .context("Failed to process search results")
}

// Bad: Missing error context
pub async fn search_tokens(query: &str) -> Result<Vec<TokenAnalytics>> {
    let results = pool.top_n("token_analytics", model, query, 10).await?;
    process_results(results)
}
```

### 4. Connection Management

```rust
// Good: Proper connection pool configuration
let pool_config = MongoPoolConfig {
    min_pool_size: 5,
    max_pool_size: 10,
    connect_timeout: Duration::from_secs(20),
};

// Bad: Hardcoded values without configuration
let client = Client::with_uri_str("mongodb://localhost").await?;
```

### 5. Vector Store Operations

```rust
// Good: Proper search parameters
let search_params = SearchParams::new()
    .exact(true)
    .num_candidates(100)
    .fields(vec!["embedding"]);

// Bad: Missing required parameters
let search_params = SearchParams::new()
    .exact(true)
    .num_candidates(100);
```

## Review Checklist

### MongoDB Integration

- [ ] Proper connection pool configuration
- [ ] Error handling with context
- [ ] Retry logic for transient failures
- [ ] Proper use of MongoDB Atlas features
- [ ] Connection string security

### Vector Store Implementation

- [ ] Proper embedding field configuration
- [ ] Search parameter completeness
- [ ] Index creation and management
- [ ] Query optimization
- [ ] Error handling for vector operations

### Code Quality

- [ ] Error handling with proper context
- [ ] Logging for important operations
- [ ] Performance considerations
- [ ] Type safety and null handling
- [ ] Documentation completeness

### Testing

- [ ] Unit tests for vector operations
- [ ] Integration tests for MongoDB
- [ ] Error case coverage
- [ ] Performance benchmarks
- [ ] Connection pool tests

## Common Issues to Watch

1. MongoDB Operations
   - Missing error context
   - Improper connection handling
   - Missing retry logic
   - Hardcoded configuration

2. Vector Store
   - Missing search parameters
   - Improper embedding handling
   - Missing index configuration
   - Inefficient queries

3. Error Handling
   - Generic error types
   - Missing error context
   - Improper error propagation
   - Missing logging

4. Performance
   - Connection pool misconfiguration
   - Missing indexes
   - Inefficient queries
   - Resource leaks

## Best Practices

### MongoDB Integration

```rust
// Connection Pool
impl MongoDbPool {
    pub async fn create_pool(config: MongoConfig) -> Result<Arc<MongoDbPool>> {
        let mut client_options = ClientOptions::parse(&config.uri).await?;
        config.pool_config.apply_to_options(&mut client_options);
        
        let client = Client::with_options(client_options)?;
        Ok(Arc::new(MongoDbPool { client, config }))
    }
}

// Error Handling
pub async fn insert_documents(docs: Vec<Document>) -> Result<()> {
    let collection = self.get_collection()?;
    collection
        .insert_many(docs)
        .await
        .context("Failed to insert documents")?;
    Ok(())
}
```

### Vector Store Operations

```rust
// Search Implementation
pub async fn search_similar(query: &str, limit: usize) -> Result<Vec<Document>> {
    let search_params = SearchParams::new()
        .exact(true)
        .num_candidates(100)
        .fields(vec!["embedding"]);

    let index = MongoDbVectorIndex::new(
        collection,
        model,
        "vector_index",
        search_params
    ).await?;

    index.top_n(query, limit).await
}
```

## Documentation Requirements

1. Function Documentation

```rust
/// Performs a vector similarity search in the token analytics collection
/// 
/// # Arguments
/// * `query` - The search query string
/// * `limit` - Maximum number of results to return
/// 
/// # Returns
/// * `Result<Vec<TokenAnalytics>>` - Search results or error with context
pub async fn search_tokens(query: &str, limit: usize) -> Result<Vec<TokenAnalytics>>
```

2. Error Documentation

```rust
/// Possible errors during vector store operations
#[derive(Error, Debug)]
pub enum VectorStoreError {
    #[error("MongoDB operation failed: {0}")]
    MongoError(#[from] mongodb::error::Error),
    
    #[error("Vector search failed: {0}")]
    SearchError(String),
    
    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}
```
