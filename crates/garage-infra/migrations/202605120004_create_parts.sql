CREATE TABLE parts (
    id UUID CONSTRAINT pk_parts PRIMARY KEY,
    name TEXT NOT NULL,
    sku TEXT NULL,
    quantity INTEGER NOT NULL DEFAULT 0,
    min_quantity INTEGER NOT NULL DEFAULT 0,
    unit_price BIGINT NOT NULL,
    currency TEXT NOT NULL,
    notes TEXT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT chk_parts_currency CHECK (currency IN ('BYN', 'USD')),
    CONSTRAINT chk_parts_status CHECK (status IN ('active', 'archived')),
    CONSTRAINT chk_parts_quantity_non_negative CHECK (quantity >= 0),
    CONSTRAINT chk_parts_min_quantity_non_negative CHECK (min_quantity >= 0),
    CONSTRAINT chk_parts_unit_price_non_negative CHECK (unit_price >= 0),
    CONSTRAINT chk_parts_updated_at_after_created_at CHECK (updated_at >= created_at)
);

CREATE UNIQUE INDEX idx_parts_sku ON parts(sku) WHERE sku IS NOT NULL;
CREATE INDEX idx_parts_status ON parts(status);
