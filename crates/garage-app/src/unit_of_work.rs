//! Transaction boundary ports for multi-aggregate use cases.
//!
//! These traits describe where the application layer needs atomic persistence,
//! but they do not define how a transaction is implemented. `garage-infra` can
//! later back them with a PostgreSQL transaction while tests can use no-op
//! in-memory implementations.

use async_trait::async_trait;

use crate::{
    AppResult, PartRepository, PaymentRepository, RepairPartRepository, RepairRepository,
    StockMovementRepository,
};

/// Unit of Work for `PaymentTransactionalService::record_payment`.
///
/// The scenario updates `Repair.paid_amount` and creates a `Payment`, so both
/// writes must eventually be committed or rolled back together by infra.
#[async_trait]
pub trait PaymentUnitOfWork: Send + Sync {
    type Repairs: RepairRepository;
    type Payments: PaymentRepository;

    fn repairs(&self) -> &Self::Repairs;
    fn payments(&self) -> &Self::Payments;

    async fn commit(&self) -> AppResult<()>;
    async fn rollback(&self) -> AppResult<()>;
}

/// Unit of Work for `RepairPartTransactionalService::use_part_in_repair`.
///
/// The scenario changes stock, creates a repair-part line and writes a stock
/// movement. The application layer defines the boundary; infra owns the real
/// transaction implementation.
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

/// Delegating implementation for shared Unit of Work handles.
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

/// Delegating implementation for shared Unit of Work handles.
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
