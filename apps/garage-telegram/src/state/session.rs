use teloxide::dispatching::dialogue::{Dialogue, InMemStorage};
use teloxide::types::MessageId;

use crate::state::{
    BookingDraft, CarDraft, ClientDraft, DialogState, PartDraft, RecordPaymentDraft,
    SetPartStockDraft, SetRepairLaborDraft, StartRepairDraft, UseRepairPartDraft,
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
    pub start_repair_draft: StartRepairDraft,
    pub record_payment_draft: RecordPaymentDraft,
    pub use_repair_part_draft: UseRepairPartDraft,
    pub set_repair_labor_draft: SetRepairLaborDraft,
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
        self.start_repair_draft.reset();
        self.record_payment_draft.reset();
        self.use_repair_part_draft.reset();
        self.set_repair_labor_draft.reset();
    }
}
