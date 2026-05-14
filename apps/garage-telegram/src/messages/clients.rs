use garage_domain::Client;

use crate::state::ClientDraft;

pub fn menu() -> &'static str {
    "Клиенты"
}

pub fn ask_name() -> &'static str {
    "Новый клиент\n\nВведите имя клиента."
}

pub fn ask_phone() -> &'static str {
    "Новый клиент\n\nВведите телефон клиента."
}

pub fn ask_notes() -> &'static str {
    "Новый клиент\n\nВведите заметку или отправьте `-`, если заметка не нужна."
}

pub fn confirm(draft: &ClientDraft) -> String {
    let name = draft.name.as_deref().unwrap_or("не указано");
    let phone = draft.phone.as_deref().unwrap_or("не указан");
    let notes = draft.notes.as_deref().unwrap_or("нет");

    format!("Проверьте данные клиента:\n\nИмя: {name}\nТелефон: {phone}\nЗаметка: {notes}")
}

pub fn created_card(client: &Client) -> String {
    let notes = client.notes().map(|notes| notes.as_str()).unwrap_or("нет");

    format!(
        "Клиент сохранен\n\nИмя: {}\nТелефон: {}\nЗаметка: {notes}",
        client.name().as_str(),
        client.phone().as_str()
    )
}
