//! Транзакционный вариант регистрации оплаты.
//!
//! Обычный `PaymentService` остается полезным для простых адаптеров и unit
//! tests. Этот сервис выполняет тот же use case через Unit of Work, чтобы infra
//! могла сохранить `Repair` и `Payment` в одной PostgreSQL-транзакции.

use garage_domain::{Payment, PaymentId};

use crate::{AppResult, PaymentRepository, PaymentUnitOfWork, RepairRepository};

use super::{common::require_repair, RecordPaymentCommand};

/// Прикладной сервис для транзакционной регистрации оплаты.
pub struct PaymentTransactionalService<Uow> {
    uow: Uow,
}

impl<Uow> PaymentTransactionalService<Uow>
where
    Uow: PaymentUnitOfWork,
{
    /// Создает сервис поверх транзакционного набора репозиториев.
    pub fn new(uow: Uow) -> Self {
        Self { uow }
    }

    /// Регистрирует оплату и фиксирует транзакционную границу.
    ///
    /// Ошибки валидации до первой записи возвращаются напрямую. Если падает
    /// сохранение или commit, сервис запрашивает rollback и возвращает исходную
    /// ошибку, потому что именно она объясняет причину отказа сценария.
    pub async fn record_payment(&self, command: RecordPaymentCommand) -> AppResult<Payment> {
        let mut repair = require_repair(self.uow.repairs(), command.repair_id).await?;
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

        if let Err(error) = self.uow.repairs().save(&repair).await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        if let Err(error) = self.uow.payments().save(&payment).await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        if let Err(error) = self.uow.commit().await {
            self.uow.rollback().await.ok();
            return Err(error);
        }

        Ok(payment)
    }
}
