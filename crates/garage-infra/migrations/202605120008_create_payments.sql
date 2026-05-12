CREATE TABLE payments (
    id UUID CONSTRAINT pk_payments PRIMARY KEY,
    repair_id UUID NOT NULL,
    amount BIGINT NOT NULL,
    currency TEXT NOT NULL,
    method TEXT NOT NULL,
    comment TEXT NULL,
    paid_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_payments_repair_id_repairs FOREIGN KEY (repair_id) REFERENCES repairs(id),
    CONSTRAINT chk_payments_amount_positive CHECK (amount > 0),
    CONSTRAINT chk_payments_currency CHECK (currency IN ('BYN', 'USD')),
    CONSTRAINT chk_payments_method CHECK (method IN ('cash', 'card', 'bank_transfer', 'crypto', 'other')),
    CONSTRAINT chk_payments_paid_at_before_created_at CHECK (paid_at <= created_at)
);

CREATE INDEX idx_payments_repair_id ON payments(repair_id);
