use std::sync::Arc;

use garage_app::AppResult;
use sqlx::{Postgres, Transaction};
use tokio::sync::{Mutex, MutexGuard};

use crate::repositories::repository_error;

/// Shared PostgreSQL transaction for transaction-aware repositories.
///
/// `PgPool::begin()` returns a transaction that is not tied to a borrowed
/// connection, so SQLx represents it as `Transaction<'static, Postgres>`.
/// Keeping that lifetime detail here prevents each Unit of Work from repeating
/// the same storage pattern.
#[derive(Clone)]
pub(crate) struct SharedPgTransaction {
    inner: Arc<Mutex<Option<Transaction<'static, Postgres>>>>,
}

impl SharedPgTransaction {
    pub(crate) async fn begin(pool: &sqlx::PgPool, operation: &'static str) -> AppResult<Self> {
        let tx = pool
            .begin()
            .await
            .map_err(|error| repository_error(operation, error))?;

        Ok(Self {
            inner: Arc::new(Mutex::new(Some(tx))),
        })
    }

    pub(crate) async fn lock(&self, operation: &'static str) -> AppResult<PgTransactionGuard<'_>> {
        Ok(PgTransactionGuard {
            operation,
            guard: self.inner.lock().await,
        })
    }

    pub(crate) async fn commit(&self, operation: &'static str) -> AppResult<()> {
        let tx = self
            .inner
            .lock()
            .await
            .take()
            .ok_or_else(|| repository_error(operation, "transaction is already finished"))?;

        tx.commit()
            .await
            .map_err(|error| repository_error(operation, error))
    }

    pub(crate) async fn rollback(&self, operation: &'static str) -> AppResult<()> {
        let Some(tx) = self.inner.lock().await.take() else {
            return Ok(());
        };

        tx.rollback()
            .await
            .map_err(|error| repository_error(operation, error))
    }
}

pub(crate) struct PgTransactionGuard<'a> {
    operation: &'static str,
    guard: MutexGuard<'a, Option<Transaction<'static, Postgres>>>,
}

impl PgTransactionGuard<'_> {
    pub(crate) fn transaction(&mut self) -> AppResult<&mut Transaction<'static, Postgres>> {
        self.guard
            .as_mut()
            .ok_or_else(|| repository_error(self.operation, "transaction is already finished"))
    }
}
