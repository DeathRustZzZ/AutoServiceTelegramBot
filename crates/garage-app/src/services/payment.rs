//! Сценарии оплат по ремонту.
//!
//! `PaymentService` координирует два доменных факта:
//! - `Repair` хранит агрегированную оплаченную сумму;
//! - `Payment` хранит одну строку истории оплаты.
//!
//! Сервис не дублирует финансовые проверки. Превышение итоговой стоимости,
//! нулевая оплата и запрет оплаты отмененного ремонта остаются в `Repair`.

use chrono::{DateTime, Utc};
use garage_domain::{Money, Payment, PaymentComment, PaymentId, PaymentMethod, RepairId};

use crate::{AppResult, PaymentRepository, RepairRepository};

use super::common::{require_payment, require_repair};

/// Команда регистрации оплаты.
///
/// Это application-layer input model, а не структура БД и не Telegram-тип.
/// `paid_at` хранит фактическое время оплаты, а `now` используется как момент
/// создания записи в системе и обновления `Repair`.
pub struct RecordPaymentCommand {
    pub repair_id: RepairId,
    pub amount: Money,
    pub method: PaymentMethod,
    pub comment: Option<PaymentComment>,
    pub paid_at: DateTime<Utc>,
    pub now: DateTime<Utc>,
}

/// Прикладной сервис для истории оплат.
pub struct PaymentService<Repairs, Payments> {
    repairs: Repairs,
    payments: Payments,
}

impl<Repairs, Payments> PaymentService<Repairs, Payments>
where
    Repairs: RepairRepository,
    Payments: PaymentRepository,
{
    /// Создает сервис оплат поверх repository ports.
    pub fn new(repairs: Repairs, payments: Payments) -> Self {
        Self { repairs, payments }
    }

    /// Регистрирует оплату по ремонту.
    ///
    /// Алгоритм:
    /// 1. Загружаем ремонт.
    /// 2. Обновляем агрегированную сумму оплаты через `Repair::record_payment`.
    /// 3. Создаем историческую строку `Payment`.
    /// 4. Сохраняем ремонт и оплату.
    ///
    /// Сейчас сохранение идет двумя repository-вызовами. Транзакционная
    /// гарантия появится позже в infra/unit-of-work, не в domain и не в этом
    /// порте.
    pub async fn record_payment(&self, command: RecordPaymentCommand) -> AppResult<Payment> {
        let mut repair = require_repair(&self.repairs, command.repair_id).await?;
        repair.record_payment(command.amount, command.now)?;

        let payment = Payment::new(
            PaymentId::new(),
            command.repair_id,
            command.amount,
            command.method,
            command.comment,
            command.paid_at,
            command.now,
        )?;

        self.repairs.save(&repair).await?;
        self.payments.save(&payment).await?;

        Ok(payment)
    }

    /// Возвращает историю оплат ремонта.
    ///
    /// Сначала проверяем существование ремонта, чтобы не отдавать пустой список
    /// для ошибочного `repair_id`.
    pub async fn list_repair_payments(&self, repair_id: RepairId) -> AppResult<Vec<Payment>> {
        require_repair(&self.repairs, repair_id).await?;
        self.payments.list_by_repair(repair_id).await
    }

    /// Возвращает конкретную оплату или `PaymentNotFound`.
    pub async fn get_payment(&self, payment_id: PaymentId) -> AppResult<Payment> {
        require_payment(&self.payments, payment_id).await
    }
}
