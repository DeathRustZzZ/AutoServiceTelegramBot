use teloxide::dispatching::dialogue::{Dialogue, InMemStorage};
use teloxide::types::MessageId;

use crate::state::{ClientDraft, DialogState};

pub type Storage = InMemStorage<SessionData>;
pub type UserDialogue = Dialogue<SessionData, Storage>;
pub type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Debug, Clone, Default)]
pub struct SessionData {
    pub dialog: DialogState,
    pub client_draft: ClientDraft,
    pub last_menu_msg_id: Option<MessageId>,
}

impl SessionData {
    pub fn reset_dialog(&mut self) {
        self.dialog = DialogState::Idle;
        self.client_draft.reset();
    }
}
