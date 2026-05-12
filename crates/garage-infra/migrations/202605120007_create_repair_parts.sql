CREATE TABLE repair_parts (
    id UUID CONSTRAINT pk_repair_parts PRIMARY KEY,
    repair_id UUID NOT NULL,
    part_id UUID NOT NULL,
    quantity INTEGER NOT NULL,
    unit_cost BIGINT NOT NULL,
    unit_price BIGINT NOT NULL,
    currency TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_repair_parts_repair_id_repairs FOREIGN KEY (repair_id) REFERENCES repairs(id),
    CONSTRAINT fk_repair_parts_part_id_parts FOREIGN KEY (part_id) REFERENCES parts(id),
    CONSTRAINT chk_repair_parts_quantity_positive CHECK (quantity > 0),
    CONSTRAINT chk_repair_parts_unit_cost_non_negative CHECK (unit_cost >= 0),
    CONSTRAINT chk_repair_parts_unit_price_non_negative CHECK (unit_price >= 0),
    CONSTRAINT chk_repair_parts_currency CHECK (currency IN ('BYN', 'USD'))
);

CREATE INDEX idx_repair_parts_repair_id ON repair_parts(repair_id);
CREATE INDEX idx_repair_parts_part_id ON repair_parts(part_id);
