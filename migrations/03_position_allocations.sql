CREATE TABLE IF NOT EXISTS position_allocations (
    id SERIAL PRIMARY KEY,
    token_address TEXT NOT NULL,
    allocation NUMERIC NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS position_allocations_token_address_idx ON position_allocations(token_address); 