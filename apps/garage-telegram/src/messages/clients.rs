use garage_domain::Client;

use crate::state::ClientDraft;

pub fn menu() -> &'static str {
    "👥 Клиенты. Выберите действие на нижней панели."
}

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

pub fn empty_list() -> &'static str {
    "Клиентов пока нет."
}

pub fn ask_search_query() -> &'static str {
    "Введите имя или телефон клиента:"
}

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

pub fn empty_search_results(query: &str) -> String {
    format!("По запросу `{query}` клиентов не найдено.")
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
    client_card(client, "Клиент сохранен")
}

pub fn client_card(client: &Client, title: &str) -> String {
    let notes = client.notes().map(|notes| notes.as_str()).unwrap_or("нет");

    format!(
        "{title}\n\n👤 {}\n📞 {}\n📝 {notes}",
        client.name().as_str(),
        client.phone().as_str()
    )
}
