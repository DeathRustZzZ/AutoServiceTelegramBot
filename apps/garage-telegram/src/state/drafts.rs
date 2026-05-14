#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClientDraft {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

impl ClientDraft {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
