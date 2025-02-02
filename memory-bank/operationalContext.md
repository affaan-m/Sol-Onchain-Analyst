# Operational Context
Last Updated: 2025-01-30

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

3. **Agent Coordination Service**
   ```rust
   pub struct AgentCoordinator {
       agents: Vec<Box<dyn Agent>>,
       message_bus: MessageBus,
       state_manager: StateManager,
   }
   ```
   - Agent lifecycle management
   - Inter-agent communication
   - State synchronization
   - Performance monitoring

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

3. **Trading Errors**
   ```rust
   #[derive(Error, Debug)]
   pub enum TradingError {
       #[error("Insufficient funds: {0}")]
       InsufficientFunds(String),
       #[error("Invalid trade: {0}")]
       InvalidTrade(String),
       #[error("Execution failed: {0}")]
       ExecutionError(String),
   }
   ```
   - Position validation
   - Balance checks
   - Transaction verification

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

3. **Compute**
   - Multi-core CPU
   - Minimum 32GB RAM
   - Load balancing
   - Auto-scaling

### Performance Requirements

1. **Latency Targets**
   ```rust
   pub struct PerformanceMetrics {
       trade_execution_ms: u64,    // Target: < 500ms
       market_data_refresh_ms: u64, // Target: < 1000ms
       signal_processing_ms: u64,   // Target: < 200ms
       db_query_ms: u64,           // Target: < 100ms
   }
   ```

2. **Throughput Requirements**
   - 1000+ market signals/second
   - 100+ trades/minute
   - 10000+ database operations/second
   - 100+ concurrent agents

3. **Resource Utilization**
   - CPU: < 70% sustained
   - Memory: < 80% usage
   - Disk I/O: < 70% utilization
   - Network: < 50% capacity

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

2. **Performance Metrics**
   - Trade execution latency
   - Market data freshness
   - Database query performance
   - Network latency

3. **Business Metrics**
   - Trade success rate
   - Agent performance
   - Portfolio returns
   - Risk exposure

### Alert Thresholds

1. **Critical Alerts**
   - Trade execution failures
   - Database connectivity issues
   - API authentication errors
   - Memory exhaustion

2. **Warning Alerts**
   - High latency
   - Elevated error rates
   - Resource utilization
   - Rate limit warnings

3. **Information Alerts**
   - Agent state changes
   - Database maintenance
   - Performance optimization
   - System updates

## Recovery Procedures

### 1. Database Recovery
```sql
-- Point-in-time recovery
SELECT * FROM market_signals
WHERE timestamp >= '2025-01-30 00:00:00'
  AND timestamp < '2025-01-30 01:00:00';

-- Reprocess failed trades
SELECT * FROM trade_executions
WHERE status = 'FAILED'
  AND execution_time > now() - interval '1 hour';
```

### 2. Service Recovery
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

### 3. Data Integrity
```rust
impl DataValidator {
    async fn validate_market_data(&self) -> Result<()> {
        // 1. Check data consistency
        // 2. Verify calculations
        // 3. Compare with backup sources
        // 4. Report discrepancies
    }
}
```

## Maintenance Procedures

### 1. Database Maintenance
- Daily backup verification
- Weekly index optimization
- Monthly data archival
- Quarterly performance review

### 2. System Updates
- Security patches
- Dependency updates
- Performance optimizations
- Feature deployments

### 3. Monitoring Updates
- Alert threshold adjustments
- Metric collection tuning
- Dashboard updates
- Log rotation