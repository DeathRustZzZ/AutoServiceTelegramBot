//! Ремонт автомобиля в автосервисе.
//!
//! Модуль связывает клиента, автомобиль, необязательную запись на обслуживание,
//! статус работ и финансовую часть ремонта. Здесь важно держать вместе два
//! набора правил:
//! 1. Жизненный цикл ремонта: работа начинается как `InProgress`, затем может
//!    быть закрыта как `Completed` или `Cancelled`. Финальные ремонты нельзя
//!    редактировать по бизнес-полям.
//! 2. Финансы ремонта: работа, цена запчастей, себестоимость запчастей и оплаты
//!    всегда выражены в одной валюте; оплаченная сумма не может превысить итог.
//!
//! `RepairDescription` и `RepairNotes` нормализуют пользовательский текст до
//! попадания в сущность. `Repair` отвечает за атомарность изменений: сначала
//! проверяются статус, валюта, суммы и даты, затем меняется состояние.
//!
//! В модели намеренно разделены `parts_price` и `parts_cost`: первое платит
//! клиент, второе является себестоимостью для сервиса. Это позволяет считать
//! ожидаемую и фактическую прибыль без отдельной бухгалтерской модели.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{BookingId, CarId, ClientId, Currency, Money, RepairId, SignedMoney};

/// Максимальная длина описания ремонта в Unicode-символах.
const MAX_REPAIR_DESCRIPTION_LEN: usize = 500;
/// Максимальная длина заметки по ремонту в Unicode-символах.
const MAX_REPAIR_NOTES_LEN: usize = 1000;

/// Текущее состояние ремонта.
///
/// Статус описывает именно жизненный цикл работ, а не оплату. Оплата живет
/// отдельно в `PaymentStatus`, потому что ремонт может быть завершен, но еще не
/// полностью оплачен.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepairStatus {
    /// Ремонт открыт, бизнес-поля еще можно менять.
    InProgress,
    /// Ремонт завершен и имеет `completed_at`.
    Completed,
    /// Ремонт отменен и не имеет `completed_at`.
    Cancelled,
}

/// Стабильное строковое представление статуса для ошибок, логов и простого UI.
impl std::fmt::Display for RepairStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepairStatus::InProgress => write!(f, "in_progress"),
            RepairStatus::Completed => write!(f, "completed"),
            RepairStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Расчетный статус оплаты ремонта.
///
/// Этот enum не хранится внутри `Repair`: он вычисляется из `paid_amount` и
/// `total_price`. Так модель не может попасть в противоречивое состояние вроде
/// `Paid` при оплаченной половине суммы.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentStatus {
    /// Оплат еще не было.
    Unpaid,
    /// Есть хотя бы одна оплата, но итоговая сумма еще не закрыта.
    PartiallyPaid,
    /// Оплаченная сумма равна итоговой стоимости ремонта.
    Paid,
}

/// Стабильное строковое представление статуса оплаты.
impl std::fmt::Display for PaymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentStatus::Unpaid => write!(f, "unpaid"),
            PaymentStatus::PartiallyPaid => write!(f, "partially_paid"),
            PaymentStatus::Paid => write!(f, "paid"),
        }
    }
}

/// Проверенное описание ремонта.
///
/// Описание обязательно: без него ремонт сложно отличить в истории клиента и
/// невозможно нормально показать мастеру или администратору. Внутренняя строка
/// закрыта, чтобы нельзя было создать пустое или слишком длинное описание в
/// обход `parse`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepairDescription(String);

impl RepairDescription {
    /// Нормализует пользовательское описание ремонта.
    ///
    /// Алгоритм:
    /// 1. Убираем внешние пробелы.
    /// 2. Пустую строку после `trim` отклоняем как `EmptyDescription`.
    /// 3. Длину считаем через `chars().count()`, чтобы лимит работал в
    ///    Unicode-символах, а не в байтах UTF-8.
    /// 4. Сохраняем уже очищенный текст.
    pub fn parse(input: &str) -> Result<Self, RepairError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Err(RepairError::EmptyDescription);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_REPAIR_DESCRIPTION_LEN {
            return Err(RepairError::DescriptionTooLong {
                max: MAX_REPAIR_DESCRIPTION_LEN,
                actual,
            });
        }

        Ok(Self(trimmed.to_string()))
    }

    /// Возвращает описание без копирования строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает описание в сохраненном виде.
impl std::fmt::Display for RepairDescription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Проверенная заметка по ремонту.
///
/// Заметка необязательна и хранится как `None`, если пользователь оставил поле
/// пустым. Это сохраняет один способ выразить отсутствие заметки и не заставляет
/// прикладной слой отличать `None` от пустой строки.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepairNotes(String);

impl RepairNotes {
    /// Нормализует пользовательскую заметку по ремонту.
    ///
    /// Пустая строка не является ошибкой: она означает отсутствие заметки.
    /// Непустая заметка ограничивается по количеству Unicode-символов.
    pub fn parse(input: &str) -> Result<Option<Self>, RepairError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_REPAIR_NOTES_LEN {
            return Err(RepairError::NotesTooLong {
                max: MAX_REPAIR_NOTES_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    /// Возвращает заметку без копирования строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает заметку без дополнительного форматирования.
impl std::fmt::Display for RepairNotes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Ремонт конкретного автомобиля клиента.
///
/// `Repair` - доменная сущность: у нее есть стабильный идентификатор, связи с
/// клиентом и автомобилем, необязательная связь с booking, управляемый статус,
/// финансовые поля и временные метки. Все поля закрыты, чтобы изменения
/// проходили через методы, которые проверяют инварианты.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repair {
    /// Стабильный идентификатор ремонта.
    id: RepairId,
    /// Клиент, которому принадлежит ремонт.
    client_id: ClientId,
    /// Автомобиль, по которому выполняется ремонт.
    car_id: CarId,
    /// Запись на обслуживание, из которой мог быть создан ремонт.
    ///
    /// Связь задается при `new` или `restore`. Методы attach/detach на текущем
    /// этапе не добавляются в domain: перепривязка требует проверки клиента,
    /// автомобиля и статуса booking, поэтому это будущий application-layer
    /// сценарий.
    booking_id: Option<BookingId>,
    /// Текущий статус работ.
    status: RepairStatus,
    /// Проверенное описание работ.
    description: RepairDescription,
    /// Стоимость работы мастера для клиента.
    labor_price: Money,
    /// Цена запчастей для клиента.
    parts_price: Money,
    /// Себестоимость запчастей для сервиса.
    parts_cost: Money,
    /// Сколько клиент уже оплатил.
    paid_amount: Money,
    /// Внутренняя заметка по ремонту.
    notes: Option<RepairNotes>,
    /// Момент начала ремонта.
    started_at: DateTime<Utc>,
    /// Момент завершения, только для `Completed`.
    completed_at: Option<DateTime<Utc>>,
    /// Момент создания записи ремонта.
    created_at: DateTime<Utc>,
    /// Момент последнего изменения ремонта.
    updated_at: DateTime<Utc>,
}

impl Repair {
    /// Создает новый ремонт в статусе `InProgress`.
    ///
    /// Финансовые поля должны быть в одной валюте. Оплаченная сумма при
    /// создании всегда равна нулю в валюте стоимости работ. Один момент `now`
    /// записывается в `started_at`, `created_at` и `updated_at`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: RepairId,
        client_id: ClientId,
        car_id: CarId,
        booking_id: Option<BookingId>,
        description: RepairDescription,
        labor_price: Money,
        parts_price: Money,
        parts_cost: Money,
        notes: Option<RepairNotes>,
        now: DateTime<Utc>,
    ) -> Result<Self, RepairError> {
        let paid_amount = Money::zero(labor_price.currency());

        Self::ensure_same_currency(&[labor_price, parts_price, parts_cost, paid_amount])?;

        Ok(Self {
            id,
            client_id,
            car_id,
            booking_id,
            status: RepairStatus::InProgress,
            description,
            labor_price,
            parts_price,
            parts_cost,
            paid_amount,
            notes,
            started_at: now,
            completed_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// Восстанавливает ремонт из сохраненного состояния.
    ///
    /// Метод предназначен для репозитория и поэтому принимает статус, оплаты и
    /// даты извне. Восстановление проверяет:
    /// 1. Все денежные поля выражены в одной валюте.
    /// 2. `created_at >= started_at`.
    /// 3. `updated_at >= created_at`.
    /// 4. Если `completed_at` есть, он не раньше `started_at`, а `updated_at`
    ///    не раньше `completed_at`.
    /// 5. Только `Completed` может иметь `completed_at`.
    /// 6. Оплата не превышает итоговую стоимость.
    ///
    /// Данные не исправляются автоматически: поврежденное состояние должно быть
    /// явно обработано приложением или миграцией.
    #[allow(clippy::too_many_arguments)]
    pub fn restore(
        id: RepairId,
        client_id: ClientId,
        car_id: CarId,
        booking_id: Option<BookingId>,
        status: RepairStatus,
        description: RepairDescription,
        labor_price: Money,
        parts_price: Money,
        parts_cost: Money,
        paid_amount: Money,
        notes: Option<RepairNotes>,
        started_at: DateTime<Utc>,
        completed_at: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, RepairError> {
        Self::ensure_same_currency(&[labor_price, parts_price, parts_cost, paid_amount])?;

        if created_at < started_at {
            return Err(RepairError::CreatedAtBeforeStartedAt);
        }

        if updated_at < created_at {
            return Err(RepairError::UpdatedAtBeforeCreatedAt);
        }

        if let Some(completed_at) = completed_at {
            if completed_at < started_at {
                return Err(RepairError::CompletedAtBeforeStartedAt);
            }

            if updated_at < completed_at {
                return Err(RepairError::UpdatedAtBeforeCompletedAt);
            }
        }

        match status {
            RepairStatus::Completed => {
                if completed_at.is_none() {
                    return Err(RepairError::CompletedRepairWithoutCompletedAt);
                }
            }
            RepairStatus::InProgress | RepairStatus::Cancelled => {
                if completed_at.is_some() {
                    return Err(RepairError::NonCompletedRepairWithCompletedAt);
                }
            }
        }

        let total_price = Self::calculate_total_price(labor_price, parts_price)?;

        if paid_amount.amount_minor() > total_price.amount_minor() {
            return Err(RepairError::PaymentExceedsTotal {
                paid: paid_amount,
                total: total_price,
            });
        }

        Ok(Self {
            id,
            client_id,
            car_id,
            booking_id,
            status,
            description,
            labor_price,
            parts_price,
            parts_cost,
            paid_amount,
            notes,
            started_at,
            completed_at,
            created_at,
            updated_at,
        })
    }

    /// Возвращает идентификатор ремонта.
    pub fn id(&self) -> RepairId {
        self.id
    }

    /// Возвращает идентификатор клиента.
    pub fn client_id(&self) -> ClientId {
        self.client_id
    }

    /// Возвращает идентификатор автомобиля.
    pub fn car_id(&self) -> CarId {
        self.car_id
    }

    /// Возвращает исходную запись на обслуживание, если ремонт создан из нее.
    pub fn booking_id(&self) -> Option<BookingId> {
        self.booking_id
    }

    /// Возвращает текущий статус ремонта.
    pub fn status(&self) -> RepairStatus {
        self.status
    }

    /// Возвращает проверенное описание ремонта.
    pub fn description(&self) -> &RepairDescription {
        &self.description
    }

    /// Возвращает стоимость работ.
    pub fn labor_price(&self) -> Money {
        self.labor_price
    }

    /// Возвращает цену запчастей для клиента.
    pub fn parts_price(&self) -> Money {
        self.parts_price
    }

    /// Возвращает себестоимость запчастей.
    pub fn parts_cost(&self) -> Money {
        self.parts_cost
    }

    /// Возвращает уже оплаченную сумму.
    pub fn paid_amount(&self) -> Money {
        self.paid_amount
    }

    /// Возвращает заметку, если она есть.
    pub fn notes(&self) -> Option<&RepairNotes> {
        self.notes.as_ref()
    }

    /// Возвращает момент начала ремонта.
    pub fn started_at(&self) -> &DateTime<Utc> {
        &self.started_at
    }

    /// Возвращает момент завершения для завершенного ремонта.
    pub fn completed_at(&self) -> Option<&DateTime<Utc>> {
        self.completed_at.as_ref()
    }

    /// Возвращает дату создания записи ремонта.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    /// Возвращает дату последнего изменения ремонта.
    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    /// Возвращает валюту ремонта.
    ///
    /// Все денежные поля обязаны иметь одну валюту, поэтому достаточно взять ее
    /// из `labor_price`.
    pub fn currency(&self) -> Currency {
        self.labor_price.currency()
    }

    /// Проверяет, открыт ли ремонт.
    pub fn is_in_progress(&self) -> bool {
        self.status == RepairStatus::InProgress
    }

    /// Проверяет, завершен ли ремонт успешно.
    pub fn is_completed(&self) -> bool {
        self.status == RepairStatus::Completed
    }

    /// Проверяет, отменен ли ремонт.
    pub fn is_cancelled(&self) -> bool {
        self.status == RepairStatus::Cancelled
    }

    /// Проверяет, закрыт ли ремонт финальным статусом.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            RepairStatus::Completed | RepairStatus::Cancelled
        )
    }

    /// Считает итоговую стоимость для клиента.
    ///
    /// Итог состоит из стоимости работ и цены запчастей для клиента. Себестоимость
    /// запчастей здесь не участвует: она нужна для расчета прибыли.
    pub fn total_price(&self) -> Result<Money, RepairError> {
        Self::calculate_total_price(self.labor_price, self.parts_price)
    }

    /// Считает остаток к оплате.
    ///
    /// При корректном состоянии `paid_amount <= total_price`, поэтому результат
    /// не должен быть отрицательным. Если данные повреждены, ошибка денег
    /// преобразуется в доменную ошибку ремонта.
    pub fn remaining_amount(&self) -> Result<Money, RepairError> {
        self.total_price()?
            .checked_sub(self.paid_amount)
            .map_err(RepairError::from)
    }

    /// Вычисляет статус оплаты из суммы оплат и итоговой стоимости.
    ///
    /// `PaymentStatus` не хранится отдельно, чтобы не было рассинхронизации
    /// между суммой оплат и enum-ом. Метод возвращает `Result`, потому что
    /// внутри считает `total_price()`, а тот использует checked-арифметику. В
    /// нормальном валидном состоянии ошибка маловероятна, но domain не должен
    /// скрывать overflow.
    pub fn payment_status(&self) -> Result<PaymentStatus, RepairError> {
        let total_price = self.total_price()?;

        if self.paid_amount.amount_minor() == 0 {
            return Ok(PaymentStatus::Unpaid);
        }

        if self.paid_amount.amount_minor() < total_price.amount_minor() {
            return Ok(PaymentStatus::PartiallyPaid);
        }

        Ok(PaymentStatus::Paid)
    }

    /// Считает фактическую прибыль по уже полученным оплатам.
    ///
    /// Формула: `paid_amount - parts_cost`. Результат может быть отрицательным,
    /// поэтому используется `SignedMoney`, а не `Money`.
    pub fn actual_profit(&self) -> Result<SignedMoney, RepairError> {
        let paid_amount = SignedMoney::from(self.paid_amount);
        let parts_cost = SignedMoney::from(self.parts_cost);

        paid_amount
            .checked_sub(parts_cost)
            .map_err(RepairError::from)
    }

    /// Считает ожидаемую прибыль, если клиент оплатит ремонт полностью.
    ///
    /// Формула: `(labor_price + parts_price) - parts_cost`.
    pub fn expected_profit(&self) -> Result<SignedMoney, RepairError> {
        let total_price = SignedMoney::from(self.total_price()?);
        let parts_cost = SignedMoney::from(self.parts_cost);

        total_price
            .checked_sub(parts_cost)
            .map_err(RepairError::from)
    }

    /// Возвращает текущую прибыль.
    ///
    /// Сейчас прибыль трактуется как фактическая, то есть по полученным оплатам.
    /// Ожидаемая прибыль доступна отдельным методом.
    pub fn profit(&self) -> Result<SignedMoney, RepairError> {
        self.actual_profit()
    }

    /// Меняет описание открытого ремонта.
    ///
    /// Финальные ремонты не редактируются по бизнес-полям: их история уже
    /// зафиксирована. Проверка статуса и времени выполняется до изменения
    /// описания, поэтому при ошибке состояние остается прежним.
    pub fn update_description(
        &mut self,
        description: RepairDescription,
        now: DateTime<Utc>,
    ) -> Result<(), RepairError> {
        self.ensure_in_progress_for_modification()?;
        self.touch(now)?;
        self.description = description;
        Ok(())
    }

    /// Меняет финансовые поля открытого ремонта.
    ///
    /// Алгоритм:
    /// 1. Ремонт должен быть `InProgress`.
    /// 2. Все новые суммы и уже оплаченная сумма должны быть в одной валюте.
    /// 3. Новый итог не может стать меньше уже оплаченной суммы.
    /// 4. Только после проверок обновляются timestamp и цены.
    pub fn update_prices(
        &mut self,
        labor_price: Money,
        parts_price: Money,
        parts_cost: Money,
        now: DateTime<Utc>,
    ) -> Result<(), RepairError> {
        self.ensure_in_progress_for_modification()?;
        Self::ensure_same_currency(&[labor_price, parts_price, parts_cost, self.paid_amount])?;

        let total_price = Self::calculate_total_price(labor_price, parts_price)?;

        if self.paid_amount.amount_minor() > total_price.amount_minor() {
            return Err(RepairError::PaymentExceedsTotal {
                paid: self.paid_amount,
                total: total_price,
            });
        }

        self.touch(now)?;
        self.labor_price = labor_price;
        self.parts_price = parts_price;
        self.parts_cost = parts_cost;
        Ok(())
    }

    /// Регистрирует оплату клиента.
    ///
    /// Оплаты можно добавлять к открытому или завершенному ремонту, но нельзя к
    /// отмененному. Нулевая оплата запрещена, валюта должна совпадать с валютой
    /// ремонта, а сумма оплат после добавления не может превысить итог.
    pub fn record_payment(&mut self, amount: Money, now: DateTime<Utc>) -> Result<(), RepairError> {
        self.ensure_can_record_payment()?;

        if amount.currency() != self.currency() {
            return Err(RepairError::CurrencyMismatch {
                expected: self.currency(),
                actual: amount.currency(),
            });
        }

        if amount.amount_minor() == 0 {
            return Err(RepairError::ZeroPayment);
        }

        let paid_amount = self
            .paid_amount
            .checked_add(amount)
            .map_err(RepairError::from)?;
        let total_price = self.total_price()?;

        if paid_amount.amount_minor() > total_price.amount_minor() {
            return Err(RepairError::PaymentExceedsTotal {
                paid: paid_amount,
                total: total_price,
            });
        }

        self.touch(now)?;
        self.paid_amount = paid_amount;
        Ok(())
    }

    /// Заменяет заметку по ремонту.
    ///
    /// Заметки можно менять и после финального статуса: это операционный
    /// комментарий, а не бизнес-факт ремонта.
    pub fn update_notes(
        &mut self,
        notes: Option<RepairNotes>,
        now: DateTime<Utc>,
    ) -> Result<(), RepairError> {
        self.touch(now)?;
        self.notes = notes;
        Ok(())
    }

    /// Удаляет заметку и фиксирует момент изменения.
    pub fn clear_notes(&mut self, now: DateTime<Utc>) -> Result<(), RepairError> {
        self.touch(now)?;
        self.notes = None;
        Ok(())
    }

    /// Закрывает ремонт как завершенный.
    ///
    /// `completed_at` одновременно становится `updated_at`. Дата завершения не
    /// может быть раньше начала ремонта.
    pub fn complete(&mut self, completed_at: DateTime<Utc>) -> Result<(), RepairError> {
        self.ensure_in_progress_for_transition(RepairStatus::Completed)?;

        if completed_at < self.started_at {
            return Err(RepairError::CompletedAtBeforeStartedAt);
        }

        self.touch(completed_at)?;
        self.status = RepairStatus::Completed;
        self.completed_at = Some(completed_at);
        Ok(())
    }

    /// Закрывает ремонт как отмененный.
    ///
    /// Отмена не имеет `completed_at`: это не завершенная работа, а закрытый
    /// сценарий без результата ремонта. Она не откатывает уже внесенные
    /// оплаты: возврат денег, сторнирование или корректировка оплаты являются
    /// отдельным application-layer сценарием. Domain только запрещает новые
    /// оплаты для `Cancelled` ремонта.
    pub fn cancel(&mut self, now: DateTime<Utc>) -> Result<(), RepairError> {
        self.ensure_in_progress_for_transition(RepairStatus::Cancelled)?;
        self.touch(now)?;
        self.status = RepairStatus::Cancelled;
        self.completed_at = None;
        Ok(())
    }

    /// Обновляет `updated_at`, сохраняя временной инвариант.
    ///
    /// `touch` сам является мутацией. Публичные операции должны выполнять все
    /// fallible-проверки до `touch`; после него не должно быть проверок,
    /// которые могут вернуть ошибку, иначе операция может потерять атомарность.
    fn touch(&mut self, now: DateTime<Utc>) -> Result<(), RepairError> {
        if now < self.created_at {
            return Err(RepairError::UpdatedAtBeforeCreatedAt);
        }

        self.updated_at = now;
        Ok(())
    }

    /// Проверяет, что бизнес-поля ремонта еще можно редактировать.
    fn ensure_in_progress_for_modification(&self) -> Result<(), RepairError> {
        if self.status != RepairStatus::InProgress {
            return Err(RepairError::CannotModifyFinalRepair {
                status: self.status,
            });
        }

        Ok(())
    }

    /// Проверяет, что текущий статус допускает выбранный финальный переход.
    fn ensure_in_progress_for_transition(&self, to: RepairStatus) -> Result<(), RepairError> {
        if self.status != RepairStatus::InProgress {
            return Err(RepairError::CannotTransitionStatus {
                from: self.status,
                to,
            });
        }

        Ok(())
    }

    /// Проверяет, что к ремонту можно добавить оплату.
    ///
    /// Завершенный ремонт может принимать оплату постфактум, отмененный - нет.
    fn ensure_can_record_payment(&self) -> Result<(), RepairError> {
        if self.status == RepairStatus::Cancelled {
            return Err(RepairError::CannotRecordPaymentForCancelledRepair);
        }

        Ok(())
    }

    /// Проверяет единую валюту всех денежных полей ремонта.
    ///
    /// Автоматическая конвертация здесь недопустима: курс, дата курса и правила
    /// округления относятся к отдельному прикладному сценарию.
    fn ensure_same_currency(values: &[Money]) -> Result<(), RepairError> {
        let Some((first, rest)) = values.split_first() else {
            return Ok(());
        };
        let expected = first.currency();

        for value in rest {
            let actual = value.currency();
            if actual != expected {
                return Err(RepairError::CurrencyMismatch { expected, actual });
            }
        }

        Ok(())
    }

    /// Считает итоговую цену как `labor_price + parts_price`.
    fn calculate_total_price(labor_price: Money, parts_price: Money) -> Result<Money, RepairError> {
        labor_price
            .checked_add(parts_price)
            .map_err(RepairError::from)
    }
}

/// Преобразует ошибки денежной арифметики в ошибки ремонта.
///
/// Это держит публичный API ремонта в терминах `RepairError`: вызывающему слою
/// не нужно знать, на каком внутреннем шаге возникла проблема.
impl From<crate::MoneyError> for RepairError {
    fn from(error: crate::MoneyError) -> Self {
        match error {
            crate::MoneyError::CurrencyMismatch { left, right } => RepairError::CurrencyMismatch {
                expected: left,
                actual: right,
            },
            crate::MoneyError::Overflow => RepairError::MoneyOverflow,
            crate::MoneyError::NegativeAmount => RepairError::NegativeMoneyResult,
        }
    }
}

/// Ошибки доменной модели ремонта.
///
/// Ошибки описывают нарушения инвариантов ремонта: некорректный текст,
/// смешанные валюты, невозможную оплату, поврежденный порядок дат или попытку
/// изменить ремонт в финальном статусе.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RepairError {
    /// Описание пустое после удаления пробелов по краям.
    #[error("repair description is empty")]
    EmptyDescription,

    /// Описание превышает лимит Unicode-символов.
    #[error("repair description is too long: max={max}, actual={actual}")]
    DescriptionTooLong { max: usize, actual: usize },

    /// Заметка превышает лимит Unicode-символов.
    #[error("repair notes are too long: max={max}, actual={actual}")]
    NotesTooLong { max: usize, actual: usize },

    /// Денежные поля ремонта используют разные валюты.
    #[error("repair currency mismatch: expected={expected:?}, actual={actual:?}")]
    CurrencyMismatch {
        expected: Currency,
        actual: Currency,
    },

    /// Сложение денежных сумм переполнило `i64`.
    #[error("repair money overflow")]
    MoneyOverflow,

    /// Денежная операция вернула отрицательный `Money`.
    #[error("repair money result cannot be negative")]
    NegativeMoneyResult,

    /// Нулевая оплата не является платежом.
    #[error("repair payment cannot be zero")]
    ZeroPayment,

    /// Сумма оплат стала больше итоговой стоимости ремонта.
    #[error("repair payment exceeds total: paid={paid}, total={total}")]
    PaymentExceedsTotal { paid: Money, total: Money },

    /// `updated_at` оказался раньше `created_at`.
    #[error("repair updated_at cannot be earlier than created_at")]
    UpdatedAtBeforeCreatedAt,

    /// `updated_at` оказался раньше `completed_at`.
    #[error("repair updated_at cannot be earlier than completed_at")]
    UpdatedAtBeforeCompletedAt,

    /// `created_at` оказался раньше начала ремонта.
    #[error("repair created_at cannot be earlier than started_at")]
    CreatedAtBeforeStartedAt,

    /// Завершение ремонта оказалось раньше начала ремонта.
    #[error("repair completed_at cannot be earlier than started_at")]
    CompletedAtBeforeStartedAt,

    /// Статус `Completed` восстановлен без даты завершения.
    #[error("completed repair must have completed_at")]
    CompletedRepairWithoutCompletedAt,

    /// Незавершенный ремонт восстановлен с датой завершения.
    #[error("non-completed repair cannot have completed_at")]
    NonCompletedRepairWithCompletedAt,

    /// Попытка изменить бизнес-поля финального ремонта.
    #[error("cannot modify repair with final status {status}")]
    CannotModifyFinalRepair { status: RepairStatus },

    /// Попытка выполнить недопустимый переход статуса.
    #[error("cannot transition repair status from {from} to {to}")]
    CannotTransitionStatus {
        from: RepairStatus,
        to: RepairStatus,
    },

    /// Попытка записать оплату по отмененному ремонту.
    #[error("cannot record payment for cancelled repair")]
    CannotRecordPaymentForCancelledRepair,
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, TimeZone, Utc};
    use uuid::Uuid;

    use super::{
        PaymentStatus, Repair, RepairDescription, RepairError, RepairNotes, RepairStatus,
        MAX_REPAIR_DESCRIPTION_LEN, MAX_REPAIR_NOTES_LEN,
    };
    use crate::{BookingId, CarId, ClientId, Currency, Money, RepairId, SignedMoney};

    fn fixed_time(seconds: i64) -> DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn repair_id() -> RepairId {
        RepairId::from_uuid(Uuid::from_u128(1))
    }

    fn client_id() -> ClientId {
        ClientId::from_uuid(Uuid::from_u128(2))
    }

    fn car_id() -> CarId {
        CarId::from_uuid(Uuid::from_u128(3))
    }

    fn booking_id() -> BookingId {
        BookingId::from_uuid(Uuid::from_u128(4))
    }

    fn byn(amount_minor: i64) -> Money {
        Money::byn_minor(amount_minor).unwrap()
    }

    fn usd(amount_minor: i64) -> Money {
        Money::usd_minor(amount_minor).unwrap()
    }

    fn description(value: &str) -> RepairDescription {
        RepairDescription::parse(value).unwrap()
    }

    fn notes(value: &str) -> RepairNotes {
        RepairNotes::parse(value).unwrap().unwrap()
    }

    fn in_progress_repair(now: DateTime<Utc>) -> Repair {
        Repair::new(
            repair_id(),
            client_id(),
            car_id(),
            Some(booking_id()),
            description("Замена сцепления"),
            byn(10_000),
            byn(5_000),
            byn(3_000),
            Some(notes("Согласовано с клиентом")),
            now,
        )
        .unwrap()
    }

    /// Строковые статусы ремонта стабильны для ошибок, логов и простого UI.
    #[test]
    fn repair_status_display_uses_stable_values() {
        assert_eq!(RepairStatus::InProgress.to_string(), "in_progress");
        assert_eq!(RepairStatus::Completed.to_string(), "completed");
        assert_eq!(RepairStatus::Cancelled.to_string(), "cancelled");
    }

    /// Статус оплаты тоже имеет стабильное строковое представление.
    #[test]
    fn payment_status_display_uses_stable_values() {
        assert_eq!(PaymentStatus::Unpaid.to_string(), "unpaid");
        assert_eq!(PaymentStatus::PartiallyPaid.to_string(), "partially_paid");
        assert_eq!(PaymentStatus::Paid.to_string(), "paid");
    }

    /// Описание очищается от внешних пробелов и сохраняется в каноническом виде.
    #[test]
    fn description_parse_trims_valid_description() {
        let description = RepairDescription::parse("  Замена масла  ").unwrap();

        assert_eq!(description.as_str(), "Замена масла");
        assert_eq!(description.to_string(), "Замена масла");
    }

    /// Строка из пробелов не должна становиться описанием ремонта.
    #[test]
    fn description_parse_rejects_empty_description_after_trim() {
        let error = RepairDescription::parse("   ").unwrap_err();

        assert_eq!(error, RepairError::EmptyDescription);
    }

    /// Лимит описания считается в Unicode-символах.
    #[test]
    fn description_parse_allows_unicode_description_at_max_length() {
        let input = "я".repeat(MAX_REPAIR_DESCRIPTION_LEN);

        let description = RepairDescription::parse(&input).unwrap();

        assert_eq!(
            description.as_str().chars().count(),
            MAX_REPAIR_DESCRIPTION_LEN
        );
    }

    /// Слишком длинное описание возвращает структурированную ошибку.
    #[test]
    fn description_parse_rejects_too_long_description() {
        let input = "a".repeat(MAX_REPAIR_DESCRIPTION_LEN + 1);

        let error = RepairDescription::parse(&input).unwrap_err();

        assert_eq!(
            error,
            RepairError::DescriptionTooLong {
                max: MAX_REPAIR_DESCRIPTION_LEN,
                actual: MAX_REPAIR_DESCRIPTION_LEN + 1,
            }
        );
    }

    /// Непустая заметка очищается от внешних пробелов.
    #[test]
    fn notes_parse_trims_non_empty_notes() {
        let notes = RepairNotes::parse("  Клиент просил старые детали  ")
            .unwrap()
            .unwrap();

        assert_eq!(notes.as_str(), "Клиент просил старые детали");
        assert_eq!(notes.to_string(), "Клиент просил старые детали");
    }

    /// Пустая заметка хранится как отсутствие значения.
    #[test]
    fn notes_parse_returns_none_for_blank_input() {
        let notes = RepairNotes::parse("\n\t ").unwrap();

        assert_eq!(notes, None);
    }

    /// Лимит заметки считается в Unicode-символах.
    #[test]
    fn notes_parse_allows_unicode_notes_at_max_length() {
        let input = "ю".repeat(MAX_REPAIR_NOTES_LEN);

        let notes = RepairNotes::parse(&input).unwrap().unwrap();

        assert_eq!(notes.as_str().chars().count(), MAX_REPAIR_NOTES_LEN);
    }

    /// Слишком длинная заметка возвращает точный `actual`.
    #[test]
    fn notes_parse_rejects_too_long_notes() {
        let input = "a".repeat(MAX_REPAIR_NOTES_LEN + 1);

        let error = RepairNotes::parse(&input).unwrap_err();

        assert_eq!(
            error,
            RepairError::NotesTooLong {
                max: MAX_REPAIR_NOTES_LEN,
                actual: MAX_REPAIR_NOTES_LEN + 1,
            }
        );
    }

    /// Новый ремонт открывается в `InProgress`, не имеет завершения и начинает
    /// оплату с нуля.
    #[test]
    fn new_creates_in_progress_repair_with_initial_financial_state() {
        let now = fixed_time(1_700_000_000);

        let repair = in_progress_repair(now);

        assert_eq!(repair.id(), repair_id());
        assert_eq!(repair.client_id(), client_id());
        assert_eq!(repair.car_id(), car_id());
        assert_eq!(repair.booking_id(), Some(booking_id()));
        assert_eq!(repair.status(), RepairStatus::InProgress);
        assert!(repair.is_in_progress());
        assert!(!repair.is_terminal());
        assert_eq!(repair.description().as_str(), "Замена сцепления");
        assert_eq!(repair.labor_price(), byn(10_000));
        assert_eq!(repair.parts_price(), byn(5_000));
        assert_eq!(repair.parts_cost(), byn(3_000));
        assert_eq!(repair.paid_amount(), Money::zero(Currency::Byn));
        assert_eq!(repair.notes().unwrap().as_str(), "Согласовано с клиентом");
        assert_eq!(*repair.started_at(), now);
        assert_eq!(repair.completed_at(), None);
        assert_eq!(*repair.created_at(), now);
        assert_eq!(*repair.updated_at(), now);
    }

    /// Все денежные поля ремонта должны быть в одной валюте.
    #[test]
    fn new_rejects_currency_mismatch() {
        let now = fixed_time(1_700_000_000);

        let error = Repair::new(
            repair_id(),
            client_id(),
            car_id(),
            None,
            description("Диагностика"),
            byn(10_000),
            usd(5_000),
            byn(3_000),
            None,
            now,
        )
        .unwrap_err();

        assert_eq!(
            error,
            RepairError::CurrencyMismatch {
                expected: Currency::Byn,
                actual: Currency::Usd,
            }
        );
    }

    /// Итог, остаток, статус оплаты и прибыль вычисляются из денежных полей.
    #[test]
    fn financial_calculations_use_prices_cost_and_payments() {
        let now = fixed_time(1_700_000_000);
        let mut repair = in_progress_repair(now);

        assert_eq!(repair.total_price().unwrap(), byn(15_000));
        assert_eq!(repair.remaining_amount().unwrap(), byn(15_000));
        assert_eq!(repair.payment_status().unwrap(), PaymentStatus::Unpaid);
        assert_eq!(
            repair.actual_profit().unwrap(),
            SignedMoney::new(-3_000, Currency::Byn)
        );
        assert_eq!(
            repair.expected_profit().unwrap(),
            SignedMoney::new(12_000, Currency::Byn)
        );

        repair
            .record_payment(byn(4_000), fixed_time(1_700_000_100))
            .unwrap();

        assert_eq!(repair.remaining_amount().unwrap(), byn(11_000));
        assert_eq!(
            repair.payment_status().unwrap(),
            PaymentStatus::PartiallyPaid
        );
        assert_eq!(
            repair.profit().unwrap(),
            SignedMoney::new(1_000, Currency::Byn)
        );
    }

    /// Статус оплаты вычисляется из текущей суммы оплат и итоговой цены.
    #[test]
    fn payment_status_returns_unpaid_partial_and_paid() {
        let now = fixed_time(1_700_000_000);
        let mut repair = in_progress_repair(now);

        assert_eq!(repair.payment_status().unwrap(), PaymentStatus::Unpaid);

        repair
            .record_payment(byn(4_000), fixed_time(1_700_000_100))
            .unwrap();

        assert_eq!(
            repair.payment_status().unwrap(),
            PaymentStatus::PartiallyPaid
        );

        repair
            .record_payment(byn(11_000), fixed_time(1_700_000_200))
            .unwrap();

        assert_eq!(repair.payment_status().unwrap(), PaymentStatus::Paid);
    }

    /// Восстановление принимает корректный завершенный ремонт с датой завершения.
    #[test]
    fn restore_accepts_valid_completed_repair() {
        let started_at = fixed_time(1_700_000_000);
        let created_at = fixed_time(1_700_000_010);
        let completed_at = fixed_time(1_700_000_100);
        let updated_at = fixed_time(1_700_000_200);

        let repair = Repair::restore(
            repair_id(),
            client_id(),
            car_id(),
            Some(booking_id()),
            RepairStatus::Completed,
            description("Замена тормозов"),
            byn(10_000),
            byn(5_000),
            byn(3_000),
            byn(15_000),
            None,
            started_at,
            Some(completed_at),
            created_at,
            updated_at,
        )
        .unwrap();

        assert_eq!(repair.status(), RepairStatus::Completed);
        assert!(repair.is_completed());
        assert!(repair.is_terminal());
        assert_eq!(repair.completed_at(), Some(&completed_at));
        assert_eq!(repair.payment_status().unwrap(), PaymentStatus::Paid);
    }

    /// `created_at` не может быть раньше фактического начала ремонта.
    #[test]
    fn restore_rejects_created_at_before_started_at() {
        let error = Repair::restore(
            repair_id(),
            client_id(),
            car_id(),
            None,
            RepairStatus::InProgress,
            description("Диагностика"),
            byn(10_000),
            byn(5_000),
            byn(3_000),
            byn(0),
            None,
            fixed_time(1_700_000_100),
            None,
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_200),
        )
        .unwrap_err();

        assert_eq!(error, RepairError::CreatedAtBeforeStartedAt);
    }

    /// Завершенный ремонт обязан иметь дату завершения.
    #[test]
    fn restore_rejects_completed_repair_without_completed_at() {
        let error = Repair::restore(
            repair_id(),
            client_id(),
            car_id(),
            None,
            RepairStatus::Completed,
            description("Диагностика"),
            byn(10_000),
            byn(5_000),
            byn(3_000),
            byn(0),
            None,
            fixed_time(1_700_000_000),
            None,
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_200),
        )
        .unwrap_err();

        assert_eq!(error, RepairError::CompletedRepairWithoutCompletedAt);
    }

    /// Незавершенный ремонт не должен иметь `completed_at`.
    #[test]
    fn restore_rejects_non_completed_repair_with_completed_at() {
        let error = Repair::restore(
            repair_id(),
            client_id(),
            car_id(),
            None,
            RepairStatus::Cancelled,
            description("Диагностика"),
            byn(10_000),
            byn(5_000),
            byn(3_000),
            byn(0),
            None,
            fixed_time(1_700_000_000),
            Some(fixed_time(1_700_000_100)),
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_200),
        )
        .unwrap_err();

        assert_eq!(error, RepairError::NonCompletedRepairWithCompletedAt);
    }

    /// Оплата при восстановлении не может превышать итог ремонта.
    #[test]
    fn restore_rejects_paid_amount_above_total() {
        let error = Repair::restore(
            repair_id(),
            client_id(),
            car_id(),
            None,
            RepairStatus::InProgress,
            description("Диагностика"),
            byn(10_000),
            byn(5_000),
            byn(3_000),
            byn(15_001),
            None,
            fixed_time(1_700_000_000),
            None,
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_200),
        )
        .unwrap_err();

        assert_eq!(
            error,
            RepairError::PaymentExceedsTotal {
                paid: byn(15_001),
                total: byn(15_000),
            }
        );
    }

    /// Описание открытого ремонта можно изменить с обновлением timestamp.
    #[test]
    fn update_description_changes_description_and_timestamp() {
        let now = fixed_time(1_700_000_000);
        let changed_at = fixed_time(1_700_000_100);
        let mut repair = in_progress_repair(now);

        repair
            .update_description(description("Замена ремня ГРМ"), changed_at)
            .unwrap();

        assert_eq!(repair.description().as_str(), "Замена ремня ГРМ");
        assert_eq!(*repair.updated_at(), changed_at);
    }

    /// Финальный ремонт нельзя редактировать по бизнес-полям.
    #[test]
    fn update_description_rejects_final_repair_without_mutation() {
        let now = fixed_time(1_700_000_000);
        let completed_at = fixed_time(1_700_000_100);
        let changed_at = fixed_time(1_700_000_200);
        let mut repair = in_progress_repair(now);
        repair.complete(completed_at).unwrap();

        let error = repair
            .update_description(description("Новая работа"), changed_at)
            .unwrap_err();

        assert_eq!(
            error,
            RepairError::CannotModifyFinalRepair {
                status: RepairStatus::Completed,
            }
        );
        assert_eq!(repair.description().as_str(), "Замена сцепления");
        assert_eq!(*repair.updated_at(), completed_at);
    }

    /// Обновление цен меняет все финансовые поля открытого ремонта.
    #[test]
    fn update_prices_replaces_prices_and_cost() {
        let now = fixed_time(1_700_000_000);
        let changed_at = fixed_time(1_700_000_100);
        let mut repair = in_progress_repair(now);

        repair
            .update_prices(byn(12_000), byn(6_000), byn(4_000), changed_at)
            .unwrap();

        assert_eq!(repair.labor_price(), byn(12_000));
        assert_eq!(repair.parts_price(), byn(6_000));
        assert_eq!(repair.parts_cost(), byn(4_000));
        assert_eq!(repair.total_price().unwrap(), byn(18_000));
        assert_eq!(*repair.updated_at(), changed_at);
    }

    /// Цены нельзя уменьшить ниже уже оплаченной суммы; при ошибке состояние не
    /// должно поменяться.
    #[test]
    fn update_prices_rejects_total_below_paid_amount_without_mutation() {
        let now = fixed_time(1_700_000_000);
        let paid_at = fixed_time(1_700_000_100);
        let changed_at = fixed_time(1_700_000_200);
        let mut repair = in_progress_repair(now);
        repair.record_payment(byn(12_000), paid_at).unwrap();

        let error = repair
            .update_prices(byn(8_000), byn(3_000), byn(2_000), changed_at)
            .unwrap_err();

        assert_eq!(
            error,
            RepairError::PaymentExceedsTotal {
                paid: byn(12_000),
                total: byn(11_000),
            }
        );
        assert_eq!(repair.labor_price(), byn(10_000));
        assert_eq!(repair.parts_price(), byn(5_000));
        assert_eq!(repair.parts_cost(), byn(3_000));
        assert_eq!(*repair.updated_at(), paid_at);
    }

    /// Оплаты накапливаются до полного закрытия суммы.
    #[test]
    fn record_payment_accumulates_amount_and_reaches_paid_status() {
        let now = fixed_time(1_700_000_000);
        let first_payment_at = fixed_time(1_700_000_100);
        let second_payment_at = fixed_time(1_700_000_200);
        let mut repair = in_progress_repair(now);

        repair.record_payment(byn(4_000), first_payment_at).unwrap();
        repair
            .record_payment(byn(11_000), second_payment_at)
            .unwrap();

        assert_eq!(repair.paid_amount(), byn(15_000));
        assert_eq!(repair.remaining_amount().unwrap(), byn(0));
        assert_eq!(repair.payment_status().unwrap(), PaymentStatus::Paid);
        assert_eq!(*repair.updated_at(), second_payment_at);
    }

    /// Нулевая оплата отклоняется до изменения timestamp.
    #[test]
    fn record_payment_rejects_zero_without_mutation() {
        let now = fixed_time(1_700_000_000);
        let paid_at = fixed_time(1_700_000_100);
        let mut repair = in_progress_repair(now);

        let error = repair.record_payment(byn(0), paid_at).unwrap_err();

        assert_eq!(error, RepairError::ZeroPayment);
        assert_eq!(repair.paid_amount(), byn(0));
        assert_eq!(*repair.updated_at(), now);
    }

    /// Оплата в другой валюте запрещена.
    #[test]
    fn record_payment_rejects_currency_mismatch() {
        let now = fixed_time(1_700_000_000);
        let paid_at = fixed_time(1_700_000_100);
        let mut repair = in_progress_repair(now);

        let error = repair.record_payment(usd(100), paid_at).unwrap_err();

        assert_eq!(
            error,
            RepairError::CurrencyMismatch {
                expected: Currency::Byn,
                actual: Currency::Usd,
            }
        );
        assert_eq!(repair.paid_amount(), byn(0));
    }

    /// Сумма оплат не может превысить итог ремонта.
    #[test]
    fn record_payment_rejects_payment_above_total() {
        let now = fixed_time(1_700_000_000);
        let paid_at = fixed_time(1_700_000_100);
        let mut repair = in_progress_repair(now);

        let error = repair.record_payment(byn(15_001), paid_at).unwrap_err();

        assert_eq!(
            error,
            RepairError::PaymentExceedsTotal {
                paid: byn(15_001),
                total: byn(15_000),
            }
        );
        assert_eq!(repair.paid_amount(), byn(0));
    }

    /// Завершенный ремонт может принимать оплату постфактум.
    #[test]
    fn record_payment_is_allowed_for_completed_repair() {
        let now = fixed_time(1_700_000_000);
        let completed_at = fixed_time(1_700_000_100);
        let paid_at = fixed_time(1_700_000_200);
        let mut repair = in_progress_repair(now);
        repair.complete(completed_at).unwrap();

        repair.record_payment(byn(15_000), paid_at).unwrap();

        assert_eq!(repair.status(), RepairStatus::Completed);
        assert_eq!(repair.paid_amount(), byn(15_000));
        assert_eq!(*repair.updated_at(), paid_at);
    }

    /// Отмененный ремонт не принимает оплаты.
    #[test]
    fn record_payment_rejects_cancelled_repair() {
        let now = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let paid_at = fixed_time(1_700_000_200);
        let mut repair = in_progress_repair(now);
        repair.cancel(cancelled_at).unwrap();

        let error = repair.record_payment(byn(100), paid_at).unwrap_err();

        assert_eq!(error, RepairError::CannotRecordPaymentForCancelledRepair);
        assert_eq!(repair.paid_amount(), byn(0));
        assert_eq!(*repair.updated_at(), cancelled_at);
    }

    /// Заметки можно менять после завершения ремонта.
    #[test]
    fn notes_can_be_updated_and_cleared_after_completion() {
        let now = fixed_time(1_700_000_000);
        let completed_at = fixed_time(1_700_000_100);
        let noted_at = fixed_time(1_700_000_200);
        let cleared_at = fixed_time(1_700_000_300);
        let mut repair = in_progress_repair(now);
        repair.complete(completed_at).unwrap();

        repair
            .update_notes(Some(notes("Клиент забрал авто")), noted_at)
            .unwrap();

        assert_eq!(repair.notes().unwrap().as_str(), "Клиент забрал авто");
        assert_eq!(*repair.updated_at(), noted_at);

        repair.clear_notes(cleared_at).unwrap();

        assert_eq!(repair.notes(), None);
        assert_eq!(*repair.updated_at(), cleared_at);
    }

    /// Завершение переводит ремонт в финальный статус и записывает `completed_at`.
    #[test]
    fn complete_transitions_repair_to_completed() {
        let now = fixed_time(1_700_000_000);
        let completed_at = fixed_time(1_700_000_100);
        let mut repair = in_progress_repair(now);

        repair.complete(completed_at).unwrap();

        assert_eq!(repair.status(), RepairStatus::Completed);
        assert!(repair.is_completed());
        assert!(repair.is_terminal());
        assert_eq!(repair.completed_at(), Some(&completed_at));
        assert_eq!(*repair.updated_at(), completed_at);
    }

    /// Завершить ремонт раньше его начала нельзя.
    #[test]
    fn complete_rejects_completed_at_before_started_at_without_mutation() {
        let now = fixed_time(1_700_000_000);
        let earlier = fixed_time(1_699_999_999);
        let mut repair = in_progress_repair(now);

        let error = repair.complete(earlier).unwrap_err();

        assert_eq!(error, RepairError::CompletedAtBeforeStartedAt);
        assert_eq!(repair.status(), RepairStatus::InProgress);
        assert_eq!(repair.completed_at(), None);
        assert_eq!(*repair.updated_at(), now);
    }

    /// Отмена переводит ремонт в финальный статус без `completed_at`.
    #[test]
    fn cancel_transitions_repair_to_cancelled() {
        let now = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let mut repair = in_progress_repair(now);

        repair.cancel(cancelled_at).unwrap();

        assert_eq!(repair.status(), RepairStatus::Cancelled);
        assert!(repair.is_cancelled());
        assert!(repair.is_terminal());
        assert_eq!(repair.completed_at(), None);
        assert_eq!(*repair.updated_at(), cancelled_at);
    }

    /// Отмена сохраняет уже внесенную оплату; возврат денег остается сценарием
    /// прикладного слоя.
    #[test]
    fn cancel_preserves_existing_payment_without_completed_at() {
        let now = fixed_time(1_700_000_000);
        let paid_at = fixed_time(1_700_000_100);
        let cancelled_at = fixed_time(1_700_000_200);
        let mut repair = in_progress_repair(now);

        repair.record_payment(byn(4_000), paid_at).unwrap();
        repair.cancel(cancelled_at).unwrap();

        assert_eq!(repair.status(), RepairStatus::Cancelled);
        assert_eq!(repair.paid_amount(), byn(4_000));
        assert_eq!(repair.completed_at(), None);
        assert_eq!(*repair.updated_at(), cancelled_at);
    }

    /// Финальный ремонт нельзя перевести в другой финальный статус.
    #[test]
    fn final_repair_cannot_transition_again() {
        let now = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let completed_at = fixed_time(1_700_000_200);
        let mut repair = in_progress_repair(now);
        repair.cancel(cancelled_at).unwrap();

        let error = repair.complete(completed_at).unwrap_err();

        assert_eq!(
            error,
            RepairError::CannotTransitionStatus {
                from: RepairStatus::Cancelled,
                to: RepairStatus::Completed,
            }
        );
        assert_eq!(repair.status(), RepairStatus::Cancelled);
        assert_eq!(*repair.updated_at(), cancelled_at);
    }

    /// Некорректное время изменения не должно оставлять частично измененное состояние.
    #[test]
    fn update_rejects_now_before_created_at_without_mutation() {
        let now = fixed_time(1_700_000_000);
        let earlier = fixed_time(1_699_999_999);
        let mut repair = in_progress_repair(now);

        let error = repair
            .update_description(description("Новая работа"), earlier)
            .unwrap_err();

        assert_eq!(error, RepairError::UpdatedAtBeforeCreatedAt);
        assert_eq!(repair.description().as_str(), "Замена сцепления");
        assert_eq!(*repair.updated_at(), now);
    }

    #[test]
    fn cancelled_repair_rejects_new_payment() {
        let now = fixed_time(1_700_000_000);
        let cancelled_at = fixed_time(1_700_000_100);
        let paid_at = fixed_time(1_700_000_200);

        let mut repair = in_progress_repair(now);

        repair.cancel(cancelled_at).unwrap();

        let error = repair.record_payment(byn(1_000), paid_at).unwrap_err();

        assert_eq!(error, RepairError::CannotRecordPaymentForCancelledRepair);
        assert_eq!(repair.paid_amount(), byn(0));
        assert_eq!(*repair.updated_at(), cancelled_at);
    }
}
