CREATE TABLE stock_movements (
    id UUID CONSTRAINT pk_stock_movements PRIMARY KEY,
    part_id UUID NOT NULL,
    movement_type TEXT NOT NULL,
    quantity INTEGER NOT NULL,
    reason TEXT NOT NULL,
    comment TEXT NULL,
    occurred_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_stock_movements_part_id_parts FOREIGN KEY (part_id) REFERENCES parts(id),
    CONSTRAINT chk_stock_movements_movement_type CHECK (movement_type IN ('in', 'out', 'adjustment')),
    CONSTRAINT chk_stock_movements_reason CHECK (
        reason IN (
            'supply',
            'repair_usage',
            'return_from_repair',
            'inventory_correction',
            'manual_correction',
            'other'
        )
    ),
    CONSTRAINT chk_stock_movements_quantity_positive CHECK (quantity > 0),
    CONSTRAINT chk_stock_movements_occurred_at_before_created_at CHECK (occurred_at <= created_at)
);

CREATE INDEX idx_stock_movements_part_id ON stock_movements(part_id);
