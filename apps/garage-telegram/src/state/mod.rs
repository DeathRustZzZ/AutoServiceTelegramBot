//! Состояние Telegram dialogue.
//!
//! State слой хранит только временные данные UI: текущий шаг формы, черновики
//! пользовательского ввода и id последнего экранного сообщения. Долгоживущие
//! бизнес-данные должны находиться в `garage-app`/`garage-infra`, а не здесь.

pub mod dialog_state;
pub mod drafts;
pub mod session;

pub use dialog_state::{
    AddBookingStep, AddCarStep, AddClientStep, AddPartStep, DialogState, RecordPaymentStep,
    SetPartStockStep, SetRepairLaborStep, StartRepairStep, UseRepairPartStep,
};
pub use drafts::{
    BookingDraft, CarDraft, ClientDraft, PartDraft, RecordPaymentDraft, SetPartStockDraft,
    SetRepairLaborDraft, StartRepairDraft, UseRepairPartDraft,
};
pub use session::{HandlerResult, SessionData, Storage, UserDialogue};
