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

pub fn app_error(error: &AppError) -> String {
    match error {
        AppError::Client(_) => "Проверьте имя или заметку клиента.".to_string(),
        AppError::PhoneNumber(_) => {
            "Проверьте телефон. Используйте корректный номер, например +375291234567.".to_string()
        }
        AppError::Repository { .. } => "Не удалось сохранить данные. Попробуйте позже.".to_string(),
        other => format!("Не удалось выполнить действие: {other}"),
    }
}
