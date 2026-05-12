CREATE TABLE part_supplies (
    id UUID CONSTRAINT pk_part_supplies PRIMARY KEY,
    part_id UUID NOT NULL,
    quantity INTEGER NOT NULL,
    expected_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'expected',
    supplier TEXT NULL,
    notes TEXT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_part_supplies_part_id_parts FOREIGN KEY (part_id) REFERENCES parts(id),
    CONSTRAINT chk_part_supplies_status CHECK (status IN ('expected', 'received', 'cancelled')),
    CONSTRAINT chk_part_supplies_quantity_positive CHECK (quantity > 0),
    CONSTRAINT chk_part_supplies_updated_at_after_created_at CHECK (updated_at >= created_at)
);

CREATE INDEX idx_part_supplies_part_id ON part_supplies(part_id);
CREATE INDEX idx_part_supplies_status ON part_supplies(status);
