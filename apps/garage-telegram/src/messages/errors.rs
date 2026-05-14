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

pub fn part_not_found() -> &'static str {
    "Запчасть не найдена. Возможно, она была удалена или архивирована."
}

pub fn repair_not_found() -> &'static str {
    "Ремонт не найден. Возможно, он был удалён или устарел."
}

pub fn clients_load_failed() -> &'static str {
    "Не удалось загрузить или сохранить данные. Попробуйте позже."
}

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
        AppError::Client(_) => "Проверьте имя или заметку клиента.".to_string(),
        AppError::Car(error) => car_error(error),
        AppError::Booking(_) => "Проверьте причину обращения или заметку.".to_string(),
        AppError::Part(error) => part_error(error),
        AppError::Repair(error) => repair_error(error),
        AppError::Money(_) => {
            "Цена должна быть указана в копейках, например 2500 для 25.00 BYN.".to_string()
        }
        AppError::PhoneNumber(_) => {
            "Проверьте телефон. Используйте корректный номер, например +375291234567.".to_string()
        }
        AppError::Repository { .. } => clients_load_failed().to_string(),
        other => format!("Не удалось выполнить действие: {other}"),
    }
}

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
        garage_domain::RepairError::PaymentExceedsTotal { .. }
        | garage_domain::RepairError::CannotRecordPaymentForCancelledRepair
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
