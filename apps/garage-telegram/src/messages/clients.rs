//! Тексты клиентского раздела.
//!
//! Модуль форматирует списки, карточки и шаги формы клиента. Валидация имени,
//! телефона и заметок остается в domain/application layer.

use garage_domain::Client;

use crate::state::ClientDraft;

/// Возвращает текст меню клиентского раздела.
pub fn menu() -> &'static str {
    "👥 Клиенты. Выберите действие."
}

/// Форматирует страницу списка клиентов.
pub fn list_page(clients: &[Client], page: usize) -> String {
    let mut text = format!("👥 Клиенты — страница {}\n", page + 1);

    for (index, client) in clients.iter().enumerate() {
        text.push_str(&format!(
            "\n{}. {}\n   📞 {}\n",
            index + 1,
            client.name().as_str(),
            client.phone().as_str()
        ));
    }

    text
}

/// Возвращает текст для пустого списка клиентов.
pub fn empty_list() -> &'static str {
    "Клиентов пока нет."
}

/// Возвращает prompt поискового запроса клиента.
pub fn ask_search_query() -> &'static str {
    "Введите имя или телефон клиента:"
}

/// Форматирует результаты поиска клиентов.
pub fn search_results(query: &str, clients: &[Client]) -> String {
    let mut text = format!("Результаты поиска: {query}\n");

    for (index, client) in clients.iter().enumerate() {
        text.push_str(&format!(
            "\n{}. {}\n   📞 {}\n",
            index + 1,
            client.name().as_str(),
            client.phone().as_str()
        ));
    }

    text
}

/// Возвращает текст для пустых результатов поиска клиента.
pub fn empty_search_results(query: &str) -> String {
    format!("По запросу `{query}` клиентов не найдено.")
}

/// Возвращает prompt имени нового клиента.
pub fn ask_name() -> &'static str {
    "Новый клиент\n\nВведите имя клиента."
}

/// Возвращает prompt телефона нового клиента.
pub fn ask_phone() -> &'static str {
    "Новый клиент\n\nВведите телефон клиента."
}

/// Возвращает prompt заметки по новому клиенту.
pub fn ask_notes() -> &'static str {
    "Новый клиент\n\nВведите заметку или отправьте `-`, если заметка не нужна."
}

/// Форматирует экран подтверждения перед созданием клиента.
pub fn confirm(draft: &ClientDraft) -> String {
    let name = draft.name.as_deref().unwrap_or("не указано");
    let phone = draft.phone.as_deref().unwrap_or("не указан");
    let notes = draft.notes.as_deref().unwrap_or("нет");

    format!("Проверьте данные клиента:\n\nИмя: {name}\nТелефон: {phone}\nЗаметка: {notes}")
}

/// Форматирует карточку только что созданного клиента.
pub fn created_card(client: &Client) -> String {
    client_card(client, "Клиент сохранен")
}

/// Форматирует карточку клиента с заданным заголовком.
pub fn client_card(client: &Client, title: &str) -> String {
    let notes = client.notes().map(|notes| notes.as_str()).unwrap_or("нет");

    format!(
        "{title}\n\n👤 {}\n📞 {}\n📝 {notes}",
        client.name().as_str(),
        client.phone().as_str()
    )
}
