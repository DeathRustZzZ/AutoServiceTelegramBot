use garage_domain::{CarId, ClientId};

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
