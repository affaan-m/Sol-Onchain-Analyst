# Operational Context

Last Updated: 2024-02-23

## System Operation

### Core Services

1. **Market Data Service**

   ```rust
   pub struct MarketDataService {
       birdeye_client: BirdeyeClient,
       db_pool: PgPool,
       cache: Cache,
   }
   ```

   - Real-time price and volume monitoring
   - Historical data aggregation
   - Market trend analysis
   - Data validation and cleaning

2. **Trading Service**

   ```rust
   pub struct TradingService {
       engine: TradingEngine,
       risk_manager: RiskManager,
       solana_client: SolanaClient,
   }
   ```

   - Trade execution
   - Position management
   - Risk validation
   - Transaction signing

3. **CLI Service**

   ```rust
   pub struct CliProgress {
       progress_bar: ProgressBar,
   }
   
   impl CliProgress {
       pub fn new(msg: &str) -> Self
       pub fn finish_with_message(&self, msg: &str)
   }
   ```

   - Real-time progress tracking
   - Color-coded output
   - Visual score bars
   - Market signal display

### Error Handling Patterns

1. **Database Errors**

   ```rust
   #[derive(Error, Debug)]
   pub enum DatabaseError {
       #[error("Connection failed: {0}")]
       ConnectionError(String),
       #[error("Query failed: {0}")]
       QueryError(String),
       #[error("Data validation failed: {0}")]
       ValidationError(String),
   }
   ```

   - Connection retry logic
   - Query timeout handling
   - Data integrity checks

2. **API Errors**

   ```rust
   #[derive(Error, Debug)]
   pub enum ApiError {
       #[error("Rate limit exceeded")]
       RateLimitError,
       #[error("Authentication failed: {0}")]
       AuthError(String),
       #[error("Request failed: {0}")]
       RequestError(String),
   }
   ```

   - Rate limiting
   - Authentication handling
   - Request retries

3. **CLI Errors**

   ```rust
   #[derive(Error, Debug)]
   pub enum CliError {
       #[error("Invalid command: {0}")]
       InvalidCommand(String),
       #[error("Display error: {0}")]
       DisplayError(String),
       #[error("Progress tracking error: {0}")]
       ProgressError(String),
   }
   ```

   - Command validation
   - Display formatting
   - Progress tracking

### Infrastructure Requirements

1. **Database**
   - PostgreSQL 15+ with TimescaleDB
   - Minimum 16GB RAM
   - SSD storage
   - Regular backups
   - Connection pooling

2. **Network**
   - Low latency connection
   - Redundant connectivity
   - DDoS protection
   - SSL/TLS encryption

3. **CLI Environment**
   - Terminal with ANSI color support
   - Unicode support for progress bars
   - Minimum terminal width: 80 columns
   - Recommended terminal height: 24 lines

### Performance Requirements

1. **Latency Targets**

   ```rust
   pub struct PerformanceMetrics {
       trade_execution_ms: u64,    // Target: < 500ms
       market_data_refresh_ms: u64, // Target: < 1000ms
       signal_processing_ms: u64,   // Target: < 200ms
       db_query_ms: u64,           // Target: < 100ms
       cli_update_ms: u64,         // Target: < 50ms
   }
   ```

2. **Throughput Requirements**
   - 1000+ market signals/second
   - 100+ trades/minute
   - 10000+ database operations/second
   - 100+ concurrent agents
   - 60+ CLI updates/second

3. **Resource Utilization**
   - CPU: < 70% sustained
   - Memory: < 80% usage
   - Disk I/O: < 70% utilization
   - Network: < 50% capacity
   - Terminal I/O: < 30% capacity

## Monitoring and Alerting

### System Health Monitoring

```rust
pub struct HealthCheck {
    pub service: String,
    pub status: Status,
    pub last_check: DateTime<Utc>,
    pub metrics: HashMap<String, f64>,
}
```

1. **Service Health**
   - API availability
   - Database connectivity
   - Agent status
   - Memory usage
   - CLI responsiveness

2. **Performance Metrics**
   - Trade execution latency
   - Market data freshness
   - Database query performance
   - Network latency
   - CLI update frequency

3. **Business Metrics**
   - Trade success rate
   - Agent performance
   - Portfolio returns
   - Risk exposure
   - User interaction metrics

### Alert Thresholds

1. **Critical Alerts**
   - Trade execution failures
   - Database connectivity issues
   - API authentication errors
   - Memory exhaustion
   - CLI display failures

2. **Warning Alerts**
   - High latency
   - Elevated error rates
   - Resource utilization
   - Rate limit warnings
   - Terminal I/O issues

3. **Information Alerts**
   - Agent state changes
   - Database maintenance
   - Performance optimization
   - System updates
   - CLI version updates

## Recovery Procedures

### Service Recovery

```rust
impl RecoveryManager {
    async fn recover_service(&self) -> Result<()> {
        // 1. Stop affected service
        // 2. Verify dependencies
        // 3. Restore state
        // 4. Restart service
        // 5. Verify operation
    }
}
```

### CLI Recovery

```rust
impl CliManager {
    fn recover_display(&self) -> Result<()> {
        // 1. Clear screen
        // 2. Reset progress bars
        // 3. Redraw interface
        // 4. Verify display
        // 5. Resume updates
    }
}
```

## Maintenance Procedures

1. **Regular Maintenance**
   - Database optimization
   - Log rotation
   - Cache clearing
   - CLI cache cleanup

2. **Emergency Procedures**
   - Service restart
   - Database recovery
   - State restoration
   - CLI reset

3. **Upgrade Procedures**
   - Service updates
   - Database migrations
   - Configuration updates
   - CLI version upgrades

## BirdEye V3 API Integration

### Overview
The BirdEye V3 API integration provides enhanced token analysis capabilities with LLM-based filtering. The system continuously monitors and analyzes Solana tokens, storing valuable insights in MongoDB.

### Key Components

1. BirdeyeApi Trait
   - Token list retrieval with pagination
   - Token metadata fetching
   - Market data analysis
   - Error handling and retries

2. TokenFilterService
   - LLM-based token analysis
   - Multi-stage filtering pipeline
   - MongoDB storage integration
   - Continuous processing

### Data Flow
1. Token Discovery
   - Fetches token list from BirdEye V3 API
   - Applies LLM-generated filters
   - Paginates through results

2. Analysis Pipeline
   - Initial market data analysis
   - Token scoring and filtering
   - Metadata enrichment for promising tokens
   - Final comprehensive analysis

3. Data Storage
   - MongoDB document structure
   - Analytics data organization
   - Query optimization

### Operational Requirements

1. Environment Setup
   ```
   BIRDEYE_API_KEY=<your-key>
   OPENAI_API_KEY=<your-key>
   MONGODB_URI=<your-uri>
   MONGODB_DATABASE=<your-db>
   ```

2. Running the Service
   ```bash
   cargo run --example token_filter
   ```

3. Monitoring
   - Log levels: DEBUG for detailed operation
   - MongoDB collection: token_analytics
   - Process status in logs

### Maintenance Tasks

1. Regular Checks
   - Monitor API rate limits
   - Check MongoDB storage usage
   - Verify token analysis quality

2. Performance Tuning
   - Adjust sleep duration between runs
   - Optimize MongoDB queries
   - Fine-tune LLM prompts

### Integration Status
- Feature branch: feature/birdeye-v3-llm-filter
- Ready for production use
- All tests passing
- Documentation updated

### Merge Instructions for Matt
1. The feature (Affaan: Birdeye-V3-LLM-Filter) branch is ready to merge into main
2. All core functionality is tested and working
3. No breaking changes introduced
4. MongoDB schema is backward compatible
