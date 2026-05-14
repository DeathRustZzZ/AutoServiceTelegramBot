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

    format!(
        "Проверьте данные клиента:\n\nИмя: {name}\nТелефон: {phone}\nЗаметка: {notes}\n\nСохранение в БД будет подключено следующим этапом."
    )
}

pub fn saved_placeholder() -> &'static str {
    "Данные клиента собраны. Сохранение через ClientService будет подключено следующим этапом."
}
