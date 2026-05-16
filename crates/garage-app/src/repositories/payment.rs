//! Порт хранения оплат по ремонтам.
//!
//! История оплат читается отдельно от агрегата ремонта, но согласованность
//! `Payment` и `Repair.paid_amount` обеспечивает прикладной сценарий.

use async_trait::async_trait;
use garage_domain::{Payment, PaymentId, RepairId};
use std::sync::Arc;

use crate::AppResult;

/// Порт хранения отдельных оплат по ремонту.
///
/// `Payment` является историческим фактом оплаты. Репозиторий не пересчитывает
/// `Repair.paid_amount`: согласованное сохранение оплаты и ремонта выполняет
/// отдельный прикладной сервис.
#[async_trait]
pub trait PaymentRepository: Send + Sync {
    /// Возвращает оплату или `None`.
    async fn get(&self, id: PaymentId) -> AppResult<Option<Payment>>;
    /// Сохраняет оплату.
    async fn save(&self, payment: &Payment) -> AppResult<()>;
    /// Возвращает оплаты конкретного ремонта.
    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<Payment>>;
}

/// Делегирующая реализация для `Arc<dyn PaymentRepository>`.
#[async_trait]
impl<T> PaymentRepository for Arc<T>
where
    T: PaymentRepository + ?Sized,
{
    async fn get(&self, id: PaymentId) -> AppResult<Option<Payment>> {
        (**self).get(id).await
    }

    async fn save(&self, payment: &Payment) -> AppResult<()> {
        (**self).save(payment).await
    }

    async fn list_by_repair(&self, repair_id: RepairId) -> AppResult<Vec<Payment>> {
        (**self).list_by_repair(repair_id).await
    }
}
