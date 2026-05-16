//! Порты транзакционных границ для multi-aggregate сценариев.
//!
//! Эти traits фиксируют места, где прикладному слою нужен атомарный commit
//! нескольких агрегатов. Они намеренно не описывают реализацию транзакции:
//! `garage-infra` может обернуть их в PostgreSQL-транзакцию, а тесты могут
//! использовать in-memory реализацию без настоящего rollback.

use async_trait::async_trait;

use crate::{
    AppResult, PartRepository, PaymentRepository, RepairPartRepository, RepairRepository,
    StockMovementRepository,
};

/// Unit of Work для `PaymentTransactionalService::record_payment`.
///
/// Сценарий одновременно обновляет `Repair.paid_amount` и создает `Payment`.
/// Эти записи должны фиксироваться или откатываться вместе, иначе история
/// оплат и агрегированная сумма ремонта разойдутся.
#[async_trait]
pub trait PaymentUnitOfWork: Send + Sync {
    type Repairs: RepairRepository;
    type Payments: PaymentRepository;

    fn repairs(&self) -> &Self::Repairs;
    fn payments(&self) -> &Self::Payments;

    async fn commit(&self) -> AppResult<()>;
    async fn rollback(&self) -> AppResult<()>;
}

/// Unit of Work для `RepairPartTransactionalService::use_part_in_repair`.
///
/// Сценарий меняет складской остаток, создает строку использованной запчасти,
/// пишет движение склада и обновляет суммы ремонта. Application layer задает
/// границу атомарности, а infra отвечает за реальный механизм транзакции.
#[async_trait]
pub trait RepairPartUnitOfWork: Send + Sync {
    type Repairs: RepairRepository;
    type Parts: PartRepository;
    type RepairParts: RepairPartRepository;
    type StockMovements: StockMovementRepository;

    fn repairs(&self) -> &Self::Repairs;
    fn parts(&self) -> &Self::Parts;
    fn repair_parts(&self) -> &Self::RepairParts;
    fn stock_movements(&self) -> &Self::StockMovements;

    async fn commit(&self) -> AppResult<()>;
    async fn rollback(&self) -> AppResult<()>;
}

/// Делегирующая реализация для разделяемых Unit of Work handles.
#[async_trait]
impl<T> PaymentUnitOfWork for std::sync::Arc<T>
where
    T: PaymentUnitOfWork + ?Sized,
{
    type Repairs = T::Repairs;
    type Payments = T::Payments;

    fn repairs(&self) -> &Self::Repairs {
        (**self).repairs()
    }

    fn payments(&self) -> &Self::Payments {
        (**self).payments()
    }

    async fn commit(&self) -> AppResult<()> {
        (**self).commit().await
    }

    async fn rollback(&self) -> AppResult<()> {
        (**self).rollback().await
    }
}

/// Делегирующая реализация для разделяемых Unit of Work handles.
#[async_trait]
impl<T> RepairPartUnitOfWork for std::sync::Arc<T>
where
    T: RepairPartUnitOfWork + ?Sized,
{
    type Repairs = T::Repairs;
    type Parts = T::Parts;
    type RepairParts = T::RepairParts;
    type StockMovements = T::StockMovements;

    fn repairs(&self) -> &Self::Repairs {
        (**self).repairs()
    }

    fn parts(&self) -> &Self::Parts {
        (**self).parts()
    }

    fn repair_parts(&self) -> &Self::RepairParts {
        (**self).repair_parts()
    }

    fn stock_movements(&self) -> &Self::StockMovements {
        (**self).stock_movements()
    }

    async fn commit(&self) -> AppResult<()> {
        (**self).commit().await
    }

    async fn rollback(&self) -> AppResult<()> {
        (**self).rollback().await
    }
}
