CREATE TABLE cars (
    id UUID CONSTRAINT pk_cars PRIMARY KEY,
    client_id UUID NOT NULL,
    make TEXT NOT NULL,
    model TEXT NOT NULL,
    year SMALLINT NULL,
    license_plate TEXT NULL,
    vin CHAR(17) NULL,
    notes TEXT NULL,
    registration_document_photo_ref TEXT NULL,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_cars_client_id_clients FOREIGN KEY (client_id) REFERENCES clients(id),
    CONSTRAINT chk_cars_status CHECK (status IN ('active', 'archived')),
    CONSTRAINT chk_cars_updated_at_after_created_at CHECK (updated_at >= created_at),
    CONSTRAINT chk_cars_year_range CHECK (year IS NULL OR year BETWEEN 1900 AND 2100)
);

CREATE INDEX idx_cars_client_id ON cars(client_id);
CREATE INDEX idx_cars_status ON cars(status);
