//! Transactional variant of payment registration.
//!
//! The non-transactional `PaymentService` stays available for simple adapters
//! and tests. This service uses a Unit of Work port so infra can later execute
//! the same scenario inside one PostgreSQL transaction.

use garage_domain::{Payment, PaymentId};

use crate::{AppResult, PaymentRepository, PaymentUnitOfWork, RepairRepository};

use super::{common::require_repair, RecordPaymentCommand};

/// Application service for transactional payment recording.
pub struct PaymentTransactionalService<Uow> {
    uow: Uow,
}

impl<Uow> PaymentTransactionalService<Uow>
where
    Uow: PaymentUnitOfWork,
{
    /// Creates a service over a transactional repository bundle.
    pub fn new(uow: Uow) -> Self {
        Self { uow }
    }

    /// Registers a payment and commits the transaction boundary.
    ///
    /// Validation errors before the first write are returned directly. Once a
    /// write or commit fails, the service asks the Unit of Work to roll back and
    /// preserves the original error.
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
