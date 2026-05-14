use garage_domain::{BookingId, CarId, ClientId, PartId, RepairId};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientDraft {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CarDraft {
    pub client_id: Option<ClientId>,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<String>,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
}

impl CarDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BookingDraft {
    pub client_id: Option<ClientId>,
    pub car_id: Option<CarId>,
    pub scheduled_at: Option<String>,
    pub reason: Option<String>,
    pub notes: Option<String>,
}

impl BookingDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl ClientDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PartDraft {
    pub name: Option<String>,
    pub sku: Option<String>,
    pub quantity: Option<String>,
    pub min_quantity: Option<String>,
    pub unit_price: Option<String>,
    pub notes: Option<String>,
}

impl PartDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetPartStockDraft {
    pub part_id: Option<PartId>,
}

impl SetPartStockDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StartRepairDraft {
    pub booking_id: Option<BookingId>,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecordPaymentDraft {
    pub repair_id: Option<RepairId>,
    pub amount: Option<String>,
    pub method: Option<String>,
    pub comment: Option<String>,
}

impl RecordPaymentDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UseRepairPartDraft {
    pub repair_id: Option<RepairId>,
    pub part_id: Option<PartId>,
    pub quantity: Option<String>,
    pub unit_price: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SetRepairLaborDraft {
    pub repair_id: Option<RepairId>,
}

impl SetRepairLaborDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl UseRepairPartDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

impl StartRepairDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
