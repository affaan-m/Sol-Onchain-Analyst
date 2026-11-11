-- migrations/02_trade_status.sql
-- Drop trade_status type if exists and create custom ENUM type for trade_status
DROP TYPE IF EXISTS trade_status CASCADE;
CREATE TYPE trade_status AS ENUM (
    'open',
    'closed',
    'pending',
    'executed',
    'cancelled'
); 