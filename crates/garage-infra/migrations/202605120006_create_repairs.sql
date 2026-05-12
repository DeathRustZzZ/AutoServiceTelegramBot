CREATE TABLE repairs (
    id UUID CONSTRAINT pk_repairs PRIMARY KEY,
    client_id UUID NOT NULL,
    car_id UUID NOT NULL,
    booking_id UUID NULL,
    status TEXT NOT NULL DEFAULT 'in_progress',
    description TEXT NOT NULL,
    labor_price BIGINT NOT NULL,
    parts_price BIGINT NOT NULL,
    parts_cost BIGINT NOT NULL,
    paid_amount BIGINT NOT NULL DEFAULT 0,
    currency TEXT NOT NULL,
    notes TEXT NULL,
    started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_repairs_client_id_clients FOREIGN KEY (client_id) REFERENCES clients(id),
    CONSTRAINT fk_repairs_car_id_cars FOREIGN KEY (car_id) REFERENCES cars(id),
    CONSTRAINT fk_repairs_booking_id_bookings FOREIGN KEY (booking_id) REFERENCES bookings(id),
    CONSTRAINT chk_repairs_status CHECK (status IN ('in_progress', 'completed', 'cancelled')),
    CONSTRAINT chk_repairs_currency CHECK (currency IN ('BYN', 'USD')),
    CONSTRAINT chk_repairs_labor_price_non_negative CHECK (labor_price >= 0),
    CONSTRAINT chk_repairs_parts_price_non_negative CHECK (parts_price >= 0),
    CONSTRAINT chk_repairs_parts_cost_non_negative CHECK (parts_cost >= 0),
    CONSTRAINT chk_repairs_paid_amount_non_negative CHECK (paid_amount >= 0),
    CONSTRAINT chk_repairs_updated_at_after_created_at CHECK (updated_at >= created_at),
    CONSTRAINT chk_repairs_created_at_after_started_at CHECK (created_at >= started_at),
    CONSTRAINT chk_repairs_completed_at_matches_status CHECK (
        (status = 'completed' AND completed_at IS NOT NULL)
        OR (status <> 'completed' AND completed_at IS NULL)
    ),
    CONSTRAINT chk_repairs_completed_at_after_started_at CHECK (completed_at IS NULL OR completed_at >= started_at)
);

CREATE INDEX idx_repairs_client_id ON repairs(client_id);
CREATE INDEX idx_repairs_car_id ON repairs(car_id);
CREATE INDEX idx_repairs_status ON repairs(status);
CREATE INDEX idx_repairs_completed_at ON repairs(completed_at) WHERE status = 'completed';
