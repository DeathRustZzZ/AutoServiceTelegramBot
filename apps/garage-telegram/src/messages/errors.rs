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

pub fn car_not_found() -> &'static str {
    "Автомобиль не найден. Возможно, он был удалён или архивирован."
}

pub fn booking_not_found() -> &'static str {
    "Запись не найдена. Возможно, она была удалена или устарела."
}

pub fn clients_load_failed() -> &'static str {
    "Не удалось загрузить или сохранить данные. Попробуйте позже."
}

pub fn app_error(error: &AppError) -> String {
    match error {
        AppError::ClientNotFound(_) => client_not_found().to_string(),
        AppError::CarNotFound(_) => car_not_found().to_string(),
        AppError::BookingNotFound(_) => booking_not_found().to_string(),
        AppError::CarDoesNotBelongToClient { .. } => {
            "Этот автомобиль не принадлежит выбранному клиенту.".to_string()
        }
        AppError::Client(_) => "Проверьте имя или заметку клиента.".to_string(),
        AppError::Car(error) => car_error(error),
        AppError::Booking(_) => "Проверьте причину обращения или заметку.".to_string(),
        AppError::PhoneNumber(_) => {
            "Проверьте телефон. Используйте корректный номер, например +375291234567.".to_string()
        }
        AppError::Repository { .. } => clients_load_failed().to_string(),
        other => format!("Не удалось выполнить действие: {other}"),
    }
}

fn car_error(error: &garage_domain::CarError) -> String {
    match error {
        garage_domain::CarError::EmptyMake => "Введите марку автомобиля.".to_string(),
        garage_domain::CarError::EmptyModel => "Введите модель автомобиля.".to_string(),
        garage_domain::CarError::InvalidYear { .. } => {
            "Проверьте год выпуска. Укажите корректный год.".to_string()
        }
        garage_domain::CarError::PlateTooLong { .. } => {
            "Госномер слишком длинный. Проверьте ввод.".to_string()
        }
        garage_domain::CarError::InvalidVinLength { .. }
        | garage_domain::CarError::InvalidVinCharacters => {
            "Проверьте VIN: он должен состоять из 17 допустимых символов.".to_string()
        }
        garage_domain::CarError::MakeTooLong { .. } => "Марка слишком длинная.".to_string(),
        garage_domain::CarError::ModelTooLong { .. } => "Модель слишком длинная.".to_string(),
        garage_domain::CarError::NotesTooLong { .. } => {
            "Заметка по авто слишком длинная.".to_string()
        }
        garage_domain::CarError::UpdatedAtBeforeCreatedAt => {
            "Не удалось сохранить автомобиль. Попробуйте позже.".to_string()
        }
    }
}
