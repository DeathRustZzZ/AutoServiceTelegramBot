//! История отдельных оплат по ремонту.
//!
//! `Repair` хранит агрегированную сумму `paid_amount`, потому что ему нужно
//! быстро отвечать на вопросы "сколько оплачено" и "какой остаток". Но для
//! истории кассы этого недостаточно: клиент может платить частями, разными
//! способами и в разное время.
//!
//! `Payment` фиксирует один такой факт оплаты. Сущность намеренно не меняет
//! `Repair`: согласованное обновление ремонта и сохранение оплаты будет делать
//! application-layer use case. Так домен остается чистым, а операция с двумя
//! агрегатами позже сможет быть завернута в транзакцию инфраструктурой.

use chrono::{DateTime, Utc};
use thiserror::Error;

use crate::{Money, PaymentId, RepairId};

/// Максимальная длина комментария к оплате в Unicode-символах.
const MAX_PAYMENT_COMMENT_LEN: usize = 500;

/// Способ оплаты ремонта.
///
/// Набор вариантов намеренно небольшой и стабильный для MVP. Если позже
/// понадобится конкретизировать банки, терминалы или криптокошельки, это лучше
/// делать отдельными полями/справочниками в app/infra, а не смешивать с базовым
/// доменным фактом оплаты.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaymentMethod {
    /// Оплата наличными.
    Cash,
    /// Оплата банковской картой.
    Card,
    /// Банковский перевод.
    BankTransfer,
    /// Оплата криптовалютой.
    Crypto,
    /// Другой способ оплаты.
    Other,
}

/// Стабильное строковое представление способа оплаты для UI, логов и хранения.
impl std::fmt::Display for PaymentMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentMethod::Cash => write!(f, "cash"),
            PaymentMethod::Card => write!(f, "card"),
            PaymentMethod::BankTransfer => write!(f, "bank_transfer"),
            PaymentMethod::Crypto => write!(f, "crypto"),
            PaymentMethod::Other => write!(f, "other"),
        }
    }
}

/// Проверенный комментарий к оплате.
///
/// Комментарий необязателен. Пустой пользовательский ввод превращается в
/// `None`, чтобы в модели не было двух способов выразить отсутствие текста:
/// `None` и `Some("")`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PaymentComment(String);

impl PaymentComment {
    /// Нормализует пользовательский комментарий к оплате.
    ///
    /// Алгоритм:
    /// 1. Убираем пробелы по краям.
    /// 2. Пустой результат считаем отсутствием комментария и возвращаем
    ///    `Ok(None)`.
    /// 3. Длину считаем через `chars().count()`, чтобы кириллица и другие
    ///    Unicode-символы считались пользовательскими символами, а не байтами.
    /// 4. Сохраняем уже trimmed-строку.
    pub fn parse(input: &str) -> Result<Option<Self>, PaymentError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(None);
        }

        let actual = trimmed.chars().count();

        if actual > MAX_PAYMENT_COMMENT_LEN {
            return Err(PaymentError::CommentTooLong {
                max: MAX_PAYMENT_COMMENT_LEN,
                actual,
            });
        }

        Ok(Some(Self(trimmed.to_string())))
    }

    /// Возвращает комментарий без копирования строки.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Печатает комментарий в сохраненном trimmed-виде.
impl std::fmt::Display for PaymentComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Факт одной оплаты по конкретному ремонту.
///
/// Сущность immutable для MVP: оплата является историческим фактом. Если позже
/// понадобится исправлять ошибочно внесенную оплату, лучше добавить отдельный
/// сценарий корректировки или сторнирования, чтобы не переписывать историю без
/// следа.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Payment {
    /// Стабильный идентификатор оплаты.
    id: PaymentId,
    /// Ремонт, к которому относится оплата.
    repair_id: RepairId,
    /// Сумма оплаты. Ноль запрещен.
    amount: Money,
    /// Способ оплаты.
    method: PaymentMethod,
    /// Необязательный комментарий администратора или мастера.
    comment: Option<PaymentComment>,
    /// Фактическое время оплаты.
    paid_at: DateTime<Utc>,
    /// Момент создания записи об оплате в системе.
    created_at: DateTime<Utc>,
}

impl Payment {
    /// Создает новую оплату.
    ///
    /// Алгоритм:
    /// 1. Проверяем, что сумма не нулевая. `Money` уже защищает от
    ///    отрицательных значений, но нулевая оплата в истории не имеет
    ///    бизнес-смысла.
    /// 2. Проверяем временной порядок: фактическая оплата не может быть позже
    ///    момента создания записи в системе. Задним числом оплату внести можно.
    /// 3. Сохраняем факт без изменения связанного `Repair`.
    pub fn new(
        id: PaymentId,
        repair_id: RepairId,
        amount: Money,
        method: PaymentMethod,
        comment: Option<PaymentComment>,
        paid_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, PaymentError> {
        Self::restore(id, repair_id, amount, method, comment, paid_at, created_at)
    }

    /// Восстанавливает оплату из сохраненного состояния.
    ///
    /// `restore` проверяет те же инварианты, что и `new`: данные из хранилища
    /// не становятся валидными автоматически, если нарушают правила оплаты.
    pub fn restore(
        id: PaymentId,
        repair_id: RepairId,
        amount: Money,
        method: PaymentMethod,
        comment: Option<PaymentComment>,
        paid_at: DateTime<Utc>,
        created_at: DateTime<Utc>,
    ) -> Result<Self, PaymentError> {
        if amount.amount_minor() == 0 {
            return Err(PaymentError::ZeroAmount);
        }

        if paid_at > created_at {
            return Err(PaymentError::PaidAtAfterCreatedAt);
        }

        Ok(Self {
            id,
            repair_id,
            amount,
            method,
            comment,
            paid_at,
            created_at,
        })
    }

    /// Возвращает идентификатор оплаты.
    pub fn id(&self) -> PaymentId {
        self.id
    }

    /// Возвращает идентификатор связанного ремонта.
    pub fn repair_id(&self) -> RepairId {
        self.repair_id
    }

    /// Возвращает сумму оплаты.
    pub fn amount(&self) -> Money {
        self.amount
    }

    /// Возвращает способ оплаты.
    pub fn method(&self) -> PaymentMethod {
        self.method
    }

    /// Возвращает комментарий, если он есть.
    pub fn comment(&self) -> Option<&PaymentComment> {
        self.comment.as_ref()
    }

    /// Возвращает фактическое время оплаты.
    pub fn paid_at(&self) -> &DateTime<Utc> {
        &self.paid_at
    }

    /// Возвращает момент создания записи в системе.
    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }
}

/// Ошибка оплаты и комментария к оплате.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PaymentError {
    /// Оплата с нулевой суммой не имеет бизнес-смысла.
    #[error("payment amount cannot be zero")]
    ZeroAmount,

    /// Фактическая оплата не может быть позже создания записи в системе.
    #[error("payment paid_at cannot be later than created_at")]
    PaidAtAfterCreatedAt,

    /// Комментарий превышает допустимую длину в Unicode-символах.
    #[error("payment comment is too long: max={max}, actual={actual}")]
    CommentTooLong { max: usize, actual: usize },
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use super::{Payment, PaymentComment, PaymentError, PaymentMethod, MAX_PAYMENT_COMMENT_LEN};
    use crate::{Currency, Money, PaymentId, RepairId};

    fn fixed_time(seconds: i64) -> chrono::DateTime<Utc> {
        Utc.timestamp_opt(seconds, 0).single().unwrap()
    }

    fn payment_id() -> PaymentId {
        PaymentId::from_uuid(Uuid::from_u128(1))
    }

    fn repair_id() -> RepairId {
        RepairId::from_uuid(Uuid::from_u128(2))
    }

    fn comment(value: &str) -> PaymentComment {
        PaymentComment::parse(value).unwrap().unwrap()
    }

    #[test]
    fn payment_comment_parse_trims_non_empty_comment() {
        let comment = PaymentComment::parse("  предоплата наличными  ")
            .unwrap()
            .unwrap();

        assert_eq!(comment.as_str(), "предоплата наличными");
        assert_eq!(comment.to_string(), "предоплата наличными");
    }

    #[test]
    fn payment_comment_parse_returns_none_for_empty_input() {
        let comment = PaymentComment::parse("   ").unwrap();

        assert!(comment.is_none());
    }

    #[test]
    fn payment_comment_parse_allows_unicode_comment_at_max_length() {
        let input = "ж".repeat(MAX_PAYMENT_COMMENT_LEN);

        let comment = PaymentComment::parse(&input).unwrap().unwrap();

        assert_eq!(comment.as_str().chars().count(), MAX_PAYMENT_COMMENT_LEN);
    }

    #[test]
    fn payment_comment_parse_rejects_too_long_comment() {
        let input = "ж".repeat(MAX_PAYMENT_COMMENT_LEN + 1);

        let error = PaymentComment::parse(&input).unwrap_err();

        assert_eq!(
            error,
            PaymentError::CommentTooLong {
                max: MAX_PAYMENT_COMMENT_LEN,
                actual: MAX_PAYMENT_COMMENT_LEN + 1,
            }
        );
    }

    #[test]
    fn payment_new_accepts_valid_payment() {
        let paid_at = fixed_time(1_700_000_000);
        let created_at = fixed_time(1_700_000_100);
        let payment = Payment::new(
            payment_id(),
            repair_id(),
            Money::byn_minor(10_000).unwrap(),
            PaymentMethod::Cash,
            Some(comment("Предоплата")),
            paid_at,
            created_at,
        )
        .unwrap();

        assert_eq!(payment.id(), payment_id());
        assert_eq!(payment.repair_id(), repair_id());
        assert_eq!(payment.amount(), Money::byn_minor(10_000).unwrap());
        assert_eq!(payment.method(), PaymentMethod::Cash);
        assert_eq!(payment.comment().unwrap().as_str(), "Предоплата");
        assert_eq!(*payment.paid_at(), paid_at);
        assert_eq!(*payment.created_at(), created_at);
    }

    #[test]
    fn payment_new_rejects_zero_amount() {
        let error = Payment::new(
            payment_id(),
            repair_id(),
            Money::zero(Currency::Byn),
            PaymentMethod::Card,
            None,
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, PaymentError::ZeroAmount);
    }

    #[test]
    fn payment_new_accepts_backdated_payment() {
        let paid_at = fixed_time(1_700_000_000);
        let created_at = fixed_time(1_700_000_500);

        let payment = Payment::new(
            payment_id(),
            repair_id(),
            Money::byn_minor(5_000).unwrap(),
            PaymentMethod::BankTransfer,
            None,
            paid_at,
            created_at,
        )
        .unwrap();

        assert_eq!(*payment.paid_at(), paid_at);
        assert_eq!(*payment.created_at(), created_at);
    }

    #[test]
    fn payment_new_rejects_paid_at_after_created_at() {
        let error = Payment::new(
            payment_id(),
            repair_id(),
            Money::byn_minor(5_000).unwrap(),
            PaymentMethod::Cash,
            None,
            fixed_time(1_700_000_500),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, PaymentError::PaidAtAfterCreatedAt);
    }

    #[test]
    fn payment_restore_rejects_zero_amount() {
        let error = Payment::restore(
            payment_id(),
            repair_id(),
            Money::zero(Currency::Byn),
            PaymentMethod::Other,
            None,
            fixed_time(1_700_000_000),
            fixed_time(1_700_000_000),
        )
        .unwrap_err();

        assert_eq!(error, PaymentError::ZeroAmount);
    }

    #[test]
    fn payment_method_display_returns_stable_codes() {
        assert_eq!(PaymentMethod::Cash.to_string(), "cash");
        assert_eq!(PaymentMethod::Card.to_string(), "card");
        assert_eq!(PaymentMethod::BankTransfer.to_string(), "bank_transfer");
        assert_eq!(PaymentMethod::Crypto.to_string(), "crypto");
        assert_eq!(PaymentMethod::Other.to_string(), "other");
    }
}
