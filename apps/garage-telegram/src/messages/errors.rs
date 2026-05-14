use garage_app::AppError;

pub fn unknown_text() -> &'static str {
    "Не понял сообщение. Используйте кнопки навигации."
}

pub fn callback_without_message() -> &'static str {
    "Не удалось обновить экран: сообщение недоступно."
}

pub fn missing_client_name() -> &'static str {
    "Введите имя клиента перед сохранением."
}

pub fn missing_client_phone() -> &'static str {
    "Введите телефон клиента перед сохранением."
}

pub fn invalid_callback() -> &'static str {
    "Действие устарело или повреждено."
}

pub fn client_not_found() -> &'static str {
    "Клиент не найден. Возможно, он был удалён или архивирован."
}

pub fn clients_load_failed() -> &'static str {
    "Не удалось загрузить клиентов. Попробуйте позже."
}

pub fn app_error(error: &AppError) -> String {
    match error {
        AppError::ClientNotFound(_) => client_not_found().to_string(),
        AppError::Client(_) => "Проверьте имя или заметку клиента.".to_string(),
        AppError::PhoneNumber(_) => {
            "Проверьте телефон. Используйте корректный номер, например +375291234567.".to_string()
        }
        AppError::Repository { .. } => clients_load_failed().to_string(),
        other => format!("Не удалось выполнить действие: {other}"),
    }
}
