-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Create enum types
CREATE TYPE trade_status AS ENUM ('PENDING', 'EXECUTED', 'FAILED', 'CANCELLED');
CREATE TYPE signal_type AS ENUM ('BUY', 'SELL', 'HOLD', 'STRONG_BUY', 'STRONG_SELL', 'PRICE_SPIKE', 'PRICE_DROP', 'VOLUME_SURGE');

-- Market Signals
CREATE TABLE market_signals (
    id SERIAL,
    asset_address VARCHAR NOT NULL,
    signal_type VARCHAR NOT NULL,
    confidence DECIMAL NOT NULL,
    risk_score DECIMAL NOT NULL,
    sentiment_score DECIMAL,
    volume_change_24h DECIMAL,
    price_change_24h DECIMAL,
    timestamp TIMESTAMPTZ NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id, timestamp)
);

-- Trade Executions
CREATE TABLE trade_executions (
    id UUID DEFAULT gen_random_uuid(),
    signal_id INTEGER,
    signal_timestamp TIMESTAMPTZ,
    asset_address TEXT NOT NULL,
    size DECIMAL NOT NULL,
    entry_price DECIMAL NOT NULL,
    slippage DECIMAL NOT NULL,
    execution_time TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL,
    transaction_signature TEXT,
    fee_amount DECIMAL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (id, execution_time)
);

-- Agent Performance Metrics
CREATE TABLE agent_performance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_type TEXT NOT NULL,
    accuracy DECIMAL NOT NULL,
    total_signals INTEGER NOT NULL,
    successful_trades INTEGER NOT NULL,
    evaluation_period TSTZRANGE NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Token Analytics
CREATE TABLE token_analytics (
    id UUID DEFAULT gen_random_uuid(),
    token_address TEXT NOT NULL,
    token_name TEXT NOT NULL,
    token_symbol TEXT NOT NULL,
    price DECIMAL NOT NULL,
    volume_24h DECIMAL,
    market_cap DECIMAL,
    total_supply DECIMAL,
    holder_count INTEGER,
    metadata JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id, timestamp)
);

-- Create hypertables
SELECT create_hypertable('market_signals', 'timestamp', 
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE,
    migrate_data => TRUE
);

SELECT create_hypertable('trade_executions', 'execution_time',
    chunk_time_interval => INTERVAL '1 day',
    if_not_exists => TRUE,
    migrate_data => TRUE
);

SELECT create_hypertable('token_analytics', 'timestamp',
    chunk_time_interval => INTERVAL '1 hour',
    if_not_exists => TRUE,
    migrate_data => TRUE
);

-- Create indexes
CREATE INDEX idx_market_signals_asset_time ON market_signals(asset_address, timestamp);
CREATE INDEX idx_trade_executions_asset_time ON trade_executions(asset_address, execution_time);
CREATE INDEX idx_token_analytics_address_time ON token_analytics(token_address, timestamp);

-- Enable compression for market signals
ALTER TABLE market_signals SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'asset_address,signal_type',
    timescaledb.compress_orderby = 'timestamp'
);

-- Enable compression for trade executions
ALTER TABLE trade_executions SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'asset_address,status',
    timescaledb.compress_orderby = 'execution_time'
);

-- Enable compression for token analytics
ALTER TABLE token_analytics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'token_address',
    timescaledb.compress_orderby = 'timestamp'
);

-- Create compression policies
SELECT add_compression_policy('market_signals', INTERVAL '7 days');
SELECT add_compression_policy('trade_executions', INTERVAL '7 days');
SELECT add_compression_policy('token_analytics', INTERVAL '7 days');

-- Add retention policies
SELECT add_retention_policy('market_signals', INTERVAL '1 year');
SELECT add_retention_policy('trade_executions', INTERVAL '1 year');
SELECT add_retention_policy('token_analytics', INTERVAL '1 year');