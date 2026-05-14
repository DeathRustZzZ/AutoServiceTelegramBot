use teloxide::dispatching::dialogue::{Dialogue, InMemStorage};
use teloxide::types::MessageId;

use crate::state::{
    BookingDraft, CarDraft, ClientDraft, DialogState, PartDraft, SetPartStockDraft,
};

pub type Storage = InMemStorage<SessionData>;
pub type UserDialogue = Dialogue<SessionData, Storage>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Default)]
pub struct SessionData {
    pub dialog: DialogState,
    pub client_draft: ClientDraft,
    pub car_draft: CarDraft,
    pub booking_draft: BookingDraft,
    pub part_draft: PartDraft,
    pub set_part_stock_draft: SetPartStockDraft,
    pub last_menu_msg_id: Option<MessageId>,
}

impl SessionData {
    pub fn reset_dialog(&mut self) {
        self.dialog = DialogState::Idle;
        self.client_draft.reset();
        self.car_draft.reset();
        self.booking_draft.reset();
        self.part_draft.reset();
        self.set_part_stock_draft.reset();
    }
}
