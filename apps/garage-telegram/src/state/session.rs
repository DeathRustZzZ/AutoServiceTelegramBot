//! Session model для teloxide dialogue.
//!
//! Сессия живет в памяти процесса и подходит для MVP/одиночного инстанса бота.
//! При горизонтальном масштабировании этот тип можно оставить как контракт, а
//! storage заменить на Redis/PostgreSQL реализацию teloxide dialogue.

use teloxide::dispatching::dialogue::{Dialogue, InMemStorage};
use teloxide::types::MessageId;

use crate::state::{
    BookingDraft, CarDraft, ClientDraft, DialogState, PartDraft, RecordPaymentDraft,
    SetPartStockDraft, SetRepairLaborDraft, StartRepairDraft, UseRepairPartDraft,
};

/// In-memory storage текущей реализации dialogue.
pub type Storage = InMemStorage<SessionData>;
/// Dialogue handle, который handler использует для сохранения нового состояния.
pub type UserDialogue = Dialogue<SessionData, Storage>;
/// Единый результат Telegram handler'ов.
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

/// Все временное UI-состояние одного пользователя/чата.
///
/// `last_menu_msg_id` позволяет редактировать один экран вместо отправки новой
/// карточки на каждый шаг. Остальные поля являются черновиками активных форм.
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
    /// Возвращает пользователя в idle-состояние и очищает все черновики форм.
    ///
    /// Id последнего экранного сообщения не сбрасывается: после отмены бот
    /// продолжает переиспользовать тот же экран.
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
