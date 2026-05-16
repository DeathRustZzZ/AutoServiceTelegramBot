//! Пользовательские тексты ошибок.
//!
//! Handler'ы и прикладный слой возвращают технически точные ошибки, а этот
//! модуль переводит их в короткие сообщения для Telegram. Детали для логов
//! остаются в `handlers::errors`.

use garage_app::AppError;

/// Возвращает текст для неизвестного пользовательского сообщения.
pub fn unknown_text() -> &'static str {
    "Не понял сообщение. Используйте кнопки навигации."
}

/// Возвращает текст для callback query без доступного сообщения.
pub fn callback_without_message() -> &'static str {
    "Не удалось обновить экран: сообщение недоступно."
}

/// Возвращает текст ошибки отсутствующего имени клиента.
pub fn missing_client_name() -> &'static str {
    "Введите имя клиента перед сохранением."
}

/// Возвращает текст ошибки отсутствующего телефона клиента.
pub fn missing_client_phone() -> &'static str {
    "Введите телефон клиента перед сохранением."
}

/// Возвращает текст для поврежденной или устаревшей inline-кнопки.
pub fn invalid_callback() -> &'static str {
    "Действие устарело или повреждено."
}

/// Возвращает текст ошибки ненайденного клиента.
pub fn client_not_found() -> &'static str {
    "Клиент не найден. Возможно, он был удалён или архивирован."
}

/// Возвращает текст ошибки ненайденного автомобиля.
pub fn car_not_found() -> &'static str {
    "Автомобиль не найден. Возможно, он был удалён или архивирован."
}

/// Возвращает текст ошибки ненайденной записи.
pub fn booking_not_found() -> &'static str {
    "Запись не найдена. Возможно, она была удалена или устарела."
}

/// Возвращает текст ошибки ненайденной запчасти.
pub fn part_not_found() -> &'static str {
    "Запчасть не найдена. Возможно, она была удалена или архивирована."
}

/// Возвращает текст ошибки ненайденного ремонта.
pub fn repair_not_found() -> &'static str {
    "Ремонт не найден. Возможно, он был удалён или устарел."
}

/// Возвращает общий текст ошибки хранилища.
pub fn clients_load_failed() -> &'static str {
    "Не удалось загрузить или сохранить данные. Попробуйте позже."
}

/// Преобразует `AppError` в безопасный пользовательский текст.
///
/// Здесь намеренно не показываются SQL, operation names и внутренние id:
/// пользователь должен получить понятное действие, а инженер - подробности в
/// логах handler'а.
pub fn app_error(error: &AppError) -> String {
    match error {
        AppError::ClientNotFound(_) => client_not_found().to_string(),
        AppError::CarNotFound(_) => car_not_found().to_string(),
        AppError::BookingNotFound(_) => booking_not_found().to_string(),
        AppError::PartNotFound(_) => part_not_found().to_string(),
        AppError::RepairNotFound(_) => repair_not_found().to_string(),
        AppError::CarDoesNotBelongToClient { .. } => {
            "Этот автомобиль не принадлежит выбранному клиенту.".to_string()
        }
        AppError::CannotUsePartForClosedRepair { .. } => {
            "Нельзя добавить запчасть в завершённый или отменённый ремонт.".to_string()
        }
        AppError::Client(_) => "Проверьте имя или заметку клиента.".to_string(),
        AppError::Car(error) => car_error(error),
        AppError::Booking(_) => "Проверьте причину обращения или заметку.".to_string(),
        AppError::Part(error) => part_error(error),
        AppError::Repair(error) => repair_error(error),
        AppError::Payment(_) => "Проверьте сумму, способ оплаты или комментарий.".to_string(),
        AppError::StockMovement(_) => "Проверьте комментарий к списанию запчасти.".to_string(),
        AppError::Money(_) => "Введите сумму в BYN. Например: 50 или 50.50".to_string(),
        AppError::PhoneNumber(_) => {
            "Проверьте телефон. Используйте корректный номер, например +375291234567.".to_string()
        }
        AppError::Repository { .. } => clients_load_failed().to_string(),
        other => format!("Не удалось выполнить действие: {other}"),
    }
}

/// Преобразует ошибку домена ремонта в пользовательский текст.
fn repair_error(error: &garage_domain::RepairError) -> String {
    match error {
        garage_domain::RepairError::EmptyDescription => {
            "Проверьте описание или заметку ремонта.".to_string()
        }
        garage_domain::RepairError::DescriptionTooLong { .. }
        | garage_domain::RepairError::NotesTooLong { .. } => {
            "Проверьте описание или заметку ремонта.".to_string()
        }
        garage_domain::RepairError::CannotTransitionStatus { .. }
        | garage_domain::RepairError::CannotModifyFinalRepair { .. } => {
            "Этот ремонт уже закрыт.".to_string()
        }
        garage_domain::RepairError::PaymentExceedsTotal { .. } => {
            "Сумма оплаты больше остатка по ремонту.".to_string()
        }
        garage_domain::RepairError::CannotRecordPaymentForCancelledRepair
        | garage_domain::RepairError::ZeroPayment => {
            "Не удалось выполнить финансовую операцию по ремонту.".to_string()
        }
        garage_domain::RepairError::CurrencyMismatch { .. }
        | garage_domain::RepairError::MoneyOverflow
        | garage_domain::RepairError::NegativeMoneyResult => "Проверьте суммы ремонта.".to_string(),
        garage_domain::RepairError::UpdatedAtBeforeCreatedAt
        | garage_domain::RepairError::UpdatedAtBeforeCompletedAt
        | garage_domain::RepairError::CreatedAtBeforeStartedAt
        | garage_domain::RepairError::CompletedAtBeforeStartedAt
        | garage_domain::RepairError::CompletedRepairWithoutCompletedAt
        | garage_domain::RepairError::NonCompletedRepairWithCompletedAt => {
            "Не удалось сохранить ремонт. Попробуйте позже.".to_string()
        }
    }
}

/// Преобразует ошибку домена автомобиля в пользовательский текст.
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

/// Преобразует ошибку домена запчасти в пользовательский текст.
fn part_error(error: &garage_domain::PartError) -> String {
    match error {
        garage_domain::PartError::EmptyName => "Проверьте название запчасти.".to_string(),
        garage_domain::PartError::NameTooLong { .. } => {
            "Название запчасти слишком длинное.".to_string()
        }
        garage_domain::PartError::SkuTooLong { .. } => {
            "Проверьте SKU/артикул: значение слишком длинное.".to_string()
        }
        garage_domain::PartError::NotesTooLong { .. } => {
            "Заметка по запчасти слишком длинная.".to_string()
        }
        garage_domain::PartError::QuantityOverflow => "Количество должно быть числом.".to_string(),
        garage_domain::PartError::InsufficientStock { .. } => {
            "Недостаточно запчастей на складе.".to_string()
        }
        garage_domain::PartError::UpdatedAtBeforeCreatedAt => {
            "Не удалось сохранить запчасть. Попробуйте позже.".to_string()
        }
    }
}
