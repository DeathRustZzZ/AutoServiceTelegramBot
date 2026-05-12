CREATE TABLE bookings (
    id UUID CONSTRAINT pk_bookings PRIMARY KEY,
    client_id UUID NOT NULL,
    car_id UUID NOT NULL,
    scheduled_at TIMESTAMPTZ NOT NULL,
    status TEXT NOT NULL DEFAULT 'scheduled',
    reason TEXT NOT NULL,
    notes TEXT NULL,
    closed_at TIMESTAMPTZ NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_bookings_client_id_clients FOREIGN KEY (client_id) REFERENCES clients(id),
    CONSTRAINT fk_bookings_car_id_cars FOREIGN KEY (car_id) REFERENCES cars(id),
    CONSTRAINT chk_bookings_status CHECK (status IN ('scheduled', 'completed', 'cancelled', 'no_show')),
    CONSTRAINT chk_bookings_updated_at_after_created_at CHECK (updated_at >= created_at),
    CONSTRAINT chk_bookings_closed_at_matches_status CHECK (
        (status = 'scheduled' AND closed_at IS NULL)
        OR (status <> 'scheduled' AND closed_at IS NOT NULL)
    ),
    CONSTRAINT chk_bookings_closed_at_after_created_at CHECK (closed_at IS NULL OR closed_at >= created_at),
    CONSTRAINT chk_bookings_updated_at_after_closed_at CHECK (closed_at IS NULL OR updated_at >= closed_at)
);

CREATE INDEX idx_bookings_client_id ON bookings(client_id);
CREATE INDEX idx_bookings_car_id ON bookings(car_id);
CREATE INDEX idx_bookings_status ON bookings(status);
CREATE INDEX idx_bookings_scheduled_at ON bookings(scheduled_at) WHERE status = 'scheduled';
