use std::sync::Arc;

use garage_app::{BookingService, CarService, ClientService};
use garage_infra::db::pool::create_pool;
use garage_infra::repositories::booking::PgBookingRepository;
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppContainer {
    config: Config,
    pool: PgPool,
    clients: Arc<PgClientRepository>,
    cars: Arc<PgCarRepository>,
    bookings: Arc<PgBookingRepository>,
    client_service: Arc<ClientService<PgClientRepository>>,
    car_service: Arc<CarService<PgClientRepository, PgCarRepository>>,
    booking_service: Arc<BookingService<PgClientRepository, PgCarRepository, PgBookingRepository>>,
}

impl AppContainer {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let pool = create_pool(&config.database_url).await?;
        let clients = Arc::new(PgClientRepository::new(pool.clone()));
        let cars = Arc::new(PgCarRepository::new(pool.clone()));
        let bookings = Arc::new(PgBookingRepository::new(pool.clone()));
        let client_service = Arc::new(ClientService::new((*clients).clone()));
        let car_service = Arc::new(CarService::new((*clients).clone(), (*cars).clone()));
        let booking_service = Arc::new(BookingService::new(
            (*clients).clone(),
            (*cars).clone(),
            (*bookings).clone(),
        ));

        Ok(Self {
            config,
            pool,
            clients,
            cars,
            bookings,
            client_service,
            car_service,
            booking_service,
        })
    }

    pub fn bot_token(&self) -> &str {
        &self.config.bot_token
    }

    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }

    pub fn clients(&self) -> Arc<PgClientRepository> {
        self.clients.clone()
    }

    pub fn cars(&self) -> Arc<PgCarRepository> {
        self.cars.clone()
    }

    pub fn bookings(&self) -> Arc<PgBookingRepository> {
        self.bookings.clone()
    }

    pub fn client_service(&self) -> Arc<ClientService<PgClientRepository>> {
        self.client_service.clone()
    }

    pub fn car_service(&self) -> Arc<CarService<PgClientRepository, PgCarRepository>> {
        self.car_service.clone()
    }

    pub fn booking_service(
        &self,
    ) -> Arc<BookingService<PgClientRepository, PgCarRepository, PgBookingRepository>> {
        self.booking_service.clone()
    }

    pub fn timezone_offset_hours(&self) -> i32 {
        self.config.timezone_offset_hours
    }
}
