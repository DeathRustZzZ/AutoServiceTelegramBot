//! Сценарии записей на обслуживание.
//!
//! Booking - это план визита клиента. Деньги, работы и прибыль остаются в
//! `Repair`; здесь сервис координирует клиента, автомобиль и дату визита.

use chrono::{DateTime, Duration, Utc};
use garage_domain::{
    Booking, BookingId, BookingNotes, BookingReason, Car, CarId, Client, ClientId,
};

use crate::{AppResult, BookingRepository, CarRepository, ClientRepository};

use super::common::{ensure_car_belongs_to_client, require_booking, require_car, require_client};

/// Read model для экрана/сообщения с деталями записи.
///
/// Это application-level структура: она собирает несколько доменных агрегатов
/// для удобства UI, но не становится новой доменной сущностью.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookingDetails {
    /// Сама запись на обслуживание.
    pub booking: Booking,
    /// Клиент, которому принадлежит запись.
    pub client: Client,
    /// Автомобиль, указанный в записи.
    pub car: Car,
}

/// Application service для записей на обслуживание.
pub struct BookingService<Clients, Cars, Bookings> {
    clients: Clients,
    cars: Cars,
    bookings: Bookings,
}

impl<Clients, Cars, Bookings> BookingService<Clients, Cars, Bookings>
where
    Clients: ClientRepository,
    Cars: CarRepository,
    Bookings: BookingRepository,
{
    /// Создает сервис поверх портов клиентов, автомобилей и записей.
    pub fn new(clients: Clients, cars: Cars, bookings: Bookings) -> Self {
        Self {
            clients,
            cars,
            bookings,
        }
    }

    /// Создает новую запись для автомобиля клиента.
    ///
    /// Алгоритм:
    /// 1. Проверяем существование клиента.
    /// 2. Загружаем автомобиль.
    /// 3. Проверяем, что автомобиль принадлежит клиенту.
    /// 4. Создаем `Booking` в статусе `Scheduled`.
    /// 5. Сохраняем запись.
    pub async fn schedule_booking(
        &self,
        client_id: ClientId,
        car_id: CarId,
        scheduled_at: DateTime<Utc>,
        reason: BookingReason,
        notes: Option<BookingNotes>,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        require_client(&self.clients, client_id).await?;
        let car = require_car(&self.cars, car_id).await?;
        ensure_car_belongs_to_client(&car, client_id)?;

        let booking = Booking::new(
            BookingId::new(),
            client_id,
            car_id,
            scheduled_at,
            reason,
            notes,
            now,
        );
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    /// Переносит активную запись.
    ///
    /// Статусные правила остаются в domain: финальную запись нельзя перенести,
    /// и сервис не повторяет эту проверку вручную.
    pub async fn reschedule_booking(
        &self,
        booking_id: BookingId,
        scheduled_at: DateTime<Utc>,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.reschedule(scheduled_at, now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    /// Закрывает запись как состоявшуюся.
    ///
    /// Это не создает ремонт автоматически. Старт ремонта остается отдельным
    /// явным сценарием `RepairService::start_repair`.
    pub async fn complete_booking(
        &self,
        booking_id: BookingId,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.complete(now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    /// Отменяет запись.
    pub async fn cancel_booking(
        &self,
        booking_id: BookingId,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.cancel(now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    /// Закрывает запись как неявку клиента.
    pub async fn mark_no_show(
        &self,
        booking_id: BookingId,
        now: DateTime<Utc>,
    ) -> AppResult<Booking> {
        let mut booking = require_booking(&self.bookings, booking_id).await?;
        booking.mark_no_show(now)?;
        self.bookings.save(&booking).await?;
        Ok(booking)
    }

    /// Возвращает записи клиента после проверки, что клиент существует.
    pub async fn list_client_bookings(&self, client_id: ClientId) -> AppResult<Vec<Booking>> {
        require_client(&self.clients, client_id).await?;
        self.bookings.list_by_client(client_id).await
    }

    /// Возвращает записи автомобиля после проверки, что автомобиль существует.
    pub async fn list_car_bookings(&self, car_id: CarId) -> AppResult<Vec<Booking>> {
        require_car(&self.cars, car_id).await?;
        self.bookings.list_by_car(car_id).await
    }

    /// Возвращает запланированные записи за произвольный UTC-диапазон.
    ///
    /// Сервис не вычисляет локальную таймзону автосервиса. Telegram/UI layer
    /// может передать нужные `from/to`, а для простого MVP ниже есть UTC
    /// today/tomorrow helpers.
    pub async fn list_bookings_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Booking>> {
        self.bookings.list_scheduled_between(from, to).await
    }

    /// Возвращает записи текущего UTC-дня.
    ///
    /// Диапазон строится как `[00:00 сегодня, 00:00 завтра)`. Репозиторий
    /// отвечает за фактическую фильтрацию scheduled bookings.
    pub async fn list_today_bookings(&self, now: DateTime<Utc>) -> AppResult<Vec<Booking>> {
        let from = start_of_utc_day(now);
        let to = from + Duration::days(1);

        self.bookings.list_scheduled_between(from, to).await
    }

    /// Возвращает записи следующего UTC-дня.
    pub async fn list_tomorrow_bookings(&self, now: DateTime<Utc>) -> AppResult<Vec<Booking>> {
        let from = start_of_utc_day(now) + Duration::days(1);
        let to = from + Duration::days(1);

        self.bookings.list_scheduled_between(from, to).await
    }

    /// Загружает запись вместе с клиентом и автомобилем.
    ///
    /// Такой read model удобен Telegram UI: handler получает сразу все данные
    /// для карточки записи, не зная о порядке загрузки агрегатов.
    pub async fn get_booking_details(&self, booking_id: BookingId) -> AppResult<BookingDetails> {
        let booking = require_booking(&self.bookings, booking_id).await?;
        self.details_for_booking(booking).await
    }

    /// Возвращает детальные карточки записей за диапазон.
    ///
    /// Для каждой записи дополнительно загружаются клиент и автомобиль, а затем
    /// проверяется связь `Car -> Client`. Это защищает UI от поврежденных или
    /// несогласованных данных в хранилище.
    pub async fn list_booking_details_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<BookingDetails>> {
        let bookings = self.bookings.list_scheduled_between(from, to).await?;
        let mut details = Vec::with_capacity(bookings.len());

        for booking in bookings {
            details.push(self.details_for_booking(booking).await?);
        }

        Ok(details)
    }

    /// Собирает `BookingDetails` для одной записи.
    async fn details_for_booking(&self, booking: Booking) -> AppResult<BookingDetails> {
        let client = require_client(&self.clients, booking.client_id()).await?;
        let car = require_car(&self.cars, booking.car_id()).await?;
        ensure_car_belongs_to_client(&car, client.id())?;

        Ok(BookingDetails {
            booking,
            client,
            car,
        })
    }
}

/// Возвращает начало UTC-дня для переданного момента.
fn start_of_utc_day(value: DateTime<Utc>) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(
        value
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .expect("valid UTC midnight"),
        Utc,
    )
}
