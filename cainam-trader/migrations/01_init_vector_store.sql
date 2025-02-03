-- ensure PgVector extension is installed
CREATE EXTENSION IF NOT EXISTS vector;

-- Create table for market data
CREATE TABLE market_data (
  id uuid DEFAULT gen_random_uuid(),
  document jsonb NOT NULL,
  embedded_text text NOT NULL,
  embedding vector(1536)
);

-- Create table for trade history
CREATE TABLE trade_history (
  id uuid DEFAULT gen_random_uuid(),
  document jsonb NOT NULL,
  embedded_text text NOT NULL,
  embedding vector(1536)
);

-- Create table for risk models
CREATE TABLE risk_models (
  id uuid DEFAULT gen_random_uuid(),
  document jsonb NOT NULL,
  embedded_text text NOT NULL,
  embedding vector(1536)
);

-- Create table for sentiment analysis
CREATE TABLE sentiment_analysis (
  id uuid DEFAULT gen_random_uuid(),
  document jsonb NOT NULL,
  embedded_text text NOT NULL,
  embedding vector(1536)
);

-- Create HNSW indexes for cosine similarity search
CREATE INDEX IF NOT EXISTS market_data_embeddings_idx ON market_data
USING hnsw(embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS trade_history_embeddings_idx ON trade_history
USING hnsw(embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS risk_models_embeddings_idx ON risk_models
USING hnsw(embedding vector_cosine_ops);

CREATE INDEX IF NOT EXISTS sentiment_analysis_embeddings_idx ON sentiment_analysis
USING hnsw(embedding vector_cosine_ops); 