use garage_domain::ClientId;

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

impl ClientDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
