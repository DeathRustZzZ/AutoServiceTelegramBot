//! Сценарии чтения ремонтов.
//!
//! Query-сервис собирает данные для UI из нескольких агрегатов, но не меняет
//! доменное состояние и ничего не сохраняет в репозитории.

use garage_domain::{Car, Client, Payment, Repair, RepairId, RepairPart};

use crate::{
    AppResult, CarRepository, ClientRepository, PaymentRepository, RepairPartRepository,
    RepairRepository,
};

use super::common::{ensure_car_belongs_to_client, require_car, require_client, require_repair};

/// Детальная карточка ремонта для UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairDetails {
    pub repair: Repair,
    pub client: Client,
    pub car: Car,
    pub parts: Vec<RepairPart>,
    pub payments: Vec<Payment>,
}

/// Query-сервис для чтения детальных данных ремонта.
pub struct RepairQueryService<Clients, Cars, Repairs, RepairParts, Payments> {
    clients: Clients,
    cars: Cars,
    repairs: Repairs,
    repair_parts: RepairParts,
    payments: Payments,
}

impl<Clients, Cars, Repairs, RepairParts, Payments>
    RepairQueryService<Clients, Cars, Repairs, RepairParts, Payments>
where
    Clients: ClientRepository,
    Cars: CarRepository,
    Repairs: RepairRepository,
    RepairParts: RepairPartRepository,
    Payments: PaymentRepository,
{
    /// Создает query-сервис поверх repository ports.
    pub fn new(
        clients: Clients,
        cars: Cars,
        repairs: Repairs,
        repair_parts: RepairParts,
        payments: Payments,
    ) -> Self {
        Self {
            clients,
            cars,
            repairs,
            repair_parts,
            payments,
        }
    }

    /// Возвращает ремонт с клиентом, автомобилем, запчастями и оплатами.
    ///
    /// Архивные клиент или автомобиль здесь не запрещаются: details используются для
    /// истории и отображения уже существующих данных.
    pub async fn get_repair_details(&self, repair_id: RepairId) -> AppResult<RepairDetails> {
        let repair = require_repair(&self.repairs, repair_id).await?;
        let client = require_client(&self.clients, repair.client_id()).await?;
        let car = require_car(&self.cars, repair.car_id()).await?;
        ensure_car_belongs_to_client(&car, client.id())?;

        let parts = self.repair_parts.list_by_repair(repair_id).await?;
        let payments = self.payments.list_by_repair(repair_id).await?;

        Ok(RepairDetails {
            repair,
            client,
            car,
            parts,
            payments,
        })
    }

    /// Возвращает активные ремонты с данными клиента и автомобиля.
    pub async fn list_active_repair_details(&self) -> AppResult<Vec<RepairDetails>> {
        let repairs = self.repairs.list_active().await?;
        let mut details = Vec::with_capacity(repairs.len());

        for repair in repairs {
            details.push(self.details_for_repair(repair).await?);
        }

        Ok(details)
    }

    async fn details_for_repair(&self, repair: Repair) -> AppResult<RepairDetails> {
        let client = require_client(&self.clients, repair.client_id()).await?;
        let car = require_car(&self.cars, repair.car_id()).await?;
        ensure_car_belongs_to_client(&car, client.id())?;

        let parts = self.repair_parts.list_by_repair(repair.id()).await?;
        let payments = self.payments.list_by_repair(repair.id()).await?;

        Ok(RepairDetails {
            repair,
            client,
            car,
            parts,
            payments,
        })
    }
}
