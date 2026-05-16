//! Состояния многошаговых Telegram-диалогов.
//!
//! Каждый enum описывает не доменный статус, а позицию пользователя в форме.
//! Это важное разделение: например, `StartRepairStep::Confirm` не означает
//! созданный ремонт, а только экран подтверждения перед вызовом `garage-app`.

/// Верхнеуровневое состояние пользовательского диалога.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DialogState {
    /// Нет активной формы; текст трактуется как навигация или неизвестный ввод.
    #[default]
    Idle,
    /// Пользователь заполняет форму создания клиента.
    AddClient(AddClientStep),
    /// Пользователь добавляет автомобиль к выбранному клиенту.
    AddCar(AddCarStep),
    /// Пользователь создает запись на обслуживание.
    AddBooking(AddBookingStep),
    /// Пользователь добавляет складскую позицию.
    AddPart(AddPartStep),
    /// Пользователь запускает ремонт из записи.
    StartRepair(StartRepairStep),
    /// Пользователь регистрирует оплату ремонта.
    RecordPayment(RecordPaymentStep),
    /// Пользователь списывает складскую запчасть в ремонт.
    UseRepairPart(UseRepairPartStep),
    /// Пользователь меняет стоимость работ по ремонту.
    SetRepairLabor(SetRepairLaborStep),
    /// Пользователь вводит поисковый запрос клиента.
    SearchClient,
    /// Пользователь вводит поисковый запрос запчасти.
    SearchPart,
    /// Пользователь задает фактический складской остаток.
    SetPartStock(SetPartStockStep),
}

/// Шаги формы создания клиента.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddClientStep {
    AwaitingName,
    AwaitingPhone,
    AwaitingNotes,
    Confirm,
}

/// Шаги формы создания автомобиля.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddCarStep {
    AwaitingMake,
    AwaitingModel,
    AwaitingYear,
    AwaitingLicensePlate,
    AwaitingVin,
    Confirm,
}

/// Шаги формы создания записи на обслуживание.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddBookingStep {
    AwaitingClientSearch,
    AwaitingCarSelection,
    AwaitingDateTime,
    AwaitingReason,
    AwaitingNotes,
    Confirm,
}

/// Шаги формы создания складской позиции.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddPartStep {
    AwaitingName,
    AwaitingSku,
    AwaitingQuantity,
    AwaitingMinQuantity,
    AwaitingUnitPrice,
    AwaitingNotes,
    Confirm,
}

/// Шаги формы ручной корректировки остатка.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetPartStockStep {
    AwaitingQuantity,
}

/// Шаги формы запуска ремонта.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartRepairStep {
    AwaitingDescription,
    AwaitingNotes,
    Confirm,
}

/// Шаги формы приема оплаты.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordPaymentStep {
    AwaitingAmount,
    AwaitingMethod,
    AwaitingComment,
    Confirm,
}

/// Шаги формы списания запчасти в ремонт.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseRepairPartStep {
    AwaitingPartSearch,
    AwaitingPartSelection,
    AwaitingQuantity,
    AwaitingUnitPrice,
    AwaitingComment,
    Confirm,
}

/// Шаги формы изменения стоимости работ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetRepairLaborStep {
    AwaitingAmount,
}
