//! Сценарии ремонтов.
//!
//! Ремонт - основной финансовый агрегат системы. Сервис проверяет связи между
//! клиентом, автомобилем и опциональной записью, а финансовые инварианты
//! оставляет доменному `Repair`.

use chrono::{DateTime, Utc};
use garage_domain::{
    BookingId, CarId, ClientId, Money, Repair, RepairDescription, RepairId, RepairNotes,
};

use crate::{AppResult, BookingRepository, CarRepository, ClientRepository, RepairRepository};

use super::common::{
    ensure_booking_belongs_to_client_and_car, ensure_car_active, ensure_car_belongs_to_client,
    ensure_client_active, require_booking, require_car, require_client, require_repair,
};

/// Команда запуска ремонта.
///
/// Это не DTO базы данных и не Telegram-структура. Команда просто группирует
/// входные параметры use case, чтобы публичная сигнатура `start_repair` не
/// разрасталась и оставалась читаемой.
pub struct StartRepairCommand {
    pub client_id: ClientId,
    pub car_id: CarId,
    pub booking_id: Option<BookingId>,
    pub description: RepairDescription,
    pub labor_price: Money,
    pub parts_price: Money,
    pub parts_cost: Money,
    pub notes: Option<RepairNotes>,
    pub now: DateTime<Utc>,
}

/// Application service для ремонтов.
pub struct RepairService<Clients, Cars, Bookings, Repairs> {
    clients: Clients,
    cars: Cars,
    bookings: Bookings,
    repairs: Repairs,
}

impl<Clients, Cars, Bookings, Repairs> RepairService<Clients, Cars, Bookings, Repairs>
where
    Clients: ClientRepository,
    Cars: CarRepository,
    Bookings: BookingRepository,
    Repairs: RepairRepository,
{
    /// Создает сервис ремонтов поверх нужных repository ports.
    pub fn new(clients: Clients, cars: Cars, bookings: Bookings, repairs: Repairs) -> Self {
        Self {
            clients,
            cars,
            bookings,
            repairs,
        }
    }

    /// Запускает ремонт.
    ///
    /// Алгоритм:
    /// 1. Проверяем существование и активность клиента.
    /// 2. Загружаем автомобиль, проверяем активность и принадлежность клиенту.
    /// 3. Если указан booking, проверяем, что он относится к той же паре
    ///    `client_id + car_id`.
    /// 4. Создаем `Repair`; валюты, цены, себестоимость и начальный статус
    ///    проверяются доменом.
    /// 5. Сохраняем ремонт.
    pub async fn start_repair(&self, command: StartRepairCommand) -> AppResult<Repair> {
        let client = require_client(&self.clients, command.client_id).await?;
        ensure_client_active(&client)?;
        let car = require_car(&self.cars, command.car_id).await?;
        ensure_car_active(&car)?;
        ensure_car_belongs_to_client(&car, command.client_id)?;

        if let Some(booking_id) = command.booking_id {
            let booking = require_booking(&self.bookings, booking_id).await?;
            ensure_booking_belongs_to_client_and_car(&booking, command.client_id, command.car_id)?;
        }

        let repair = Repair::new(
            RepairId::new(),
            command.client_id,
            command.car_id,
            command.booking_id,
            command.description,
            command.labor_price,
            command.parts_price,
            command.parts_cost,
            command.notes,
            command.now,
        )?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }

    /// Регистрирует оплату по ремонту.
    ///
    /// Домен разрешает оплату для ремонта в работе и завершенного ремонта, но
    /// запрещает оплату отмененного ремонта и превышение итоговой стоимости.
    pub async fn record_payment(
        &self,
        repair_id: RepairId,
        amount: Money,
        now: DateTime<Utc>,
    ) -> AppResult<Repair> {
        let mut repair = require_repair(&self.repairs, repair_id).await?;
        repair.record_payment(amount, now)?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }

    /// Меняет стоимость работ по открытому ремонту.
    pub async fn set_labor_price(
        &self,
        repair_id: RepairId,
        labor_price: Money,
        now: DateTime<Utc>,
    ) -> AppResult<Repair> {
        let mut repair = require_repair(&self.repairs, repair_id).await?;
        repair.update_prices(labor_price, repair.parts_price(), repair.parts_cost(), now)?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }

    /// Завершает ремонт.
    ///
    /// Статусный переход и проверка `completed_at >= started_at` находятся в
    /// `Repair::complete`.
    pub async fn complete_repair(
        &self,
        repair_id: RepairId,
        completed_at: DateTime<Utc>,
    ) -> AppResult<Repair> {
        let mut repair = require_repair(&self.repairs, repair_id).await?;
        repair.complete(completed_at)?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }

    /// Отменяет ремонт.
    ///
    /// Уже внесенные оплаты не откатываются здесь. Возвраты и корректировки
    /// должны стать отдельными финансовыми сценариями app-layer.
    pub async fn cancel_repair(
        &self,
        repair_id: RepairId,
        now: DateTime<Utc>,
    ) -> AppResult<Repair> {
        let mut repair = require_repair(&self.repairs, repair_id).await?;
        repair.cancel(now)?;
        self.repairs.save(&repair).await?;
        Ok(repair)
    }
}
