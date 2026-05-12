CREATE TABLE clients (
    id UUID CONSTRAINT pk_clients PRIMARY KEY,
    name TEXT NOT NULL,
    phone TEXT NOT NULL,
    notes TEXT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT chk_clients_status CHECK (status IN ('active', 'archived')),
    CONSTRAINT chk_clients_updated_at_after_created_at CHECK (updated_at >= created_at)
);

CREATE INDEX idx_clients_status ON clients(status);
