use std::sync::Arc;

use garage_app::{
    AppResult, BookingService, CarService, ClientService, PartService, PaymentTransactionalService,
    RecordPaymentCommand, RepairPartTransactionalService, RepairQueryService, RepairService,
    UsePartInRepairCommand, UsePartInRepairResult,
};
use garage_infra::db::pool::create_pool;
use garage_infra::repositories::booking::PgBookingRepository;
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use garage_infra::repositories::part::PgPartRepository;
use garage_infra::repositories::payment::PgPaymentRepository;
use garage_infra::repositories::repair::PgRepairRepository;
use garage_infra::repositories::repair_part::PgRepairPartRepository;
use garage_infra::repositories::stock_movement::PgStockMovementRepository;
use garage_infra::unit_of_work::payment::PgPaymentUnitOfWork;
use garage_infra::unit_of_work::repair_part::PgRepairPartUnitOfWork;
use sqlx::PgPool;

use crate::config::Config;

#[derive(Clone)]
pub struct AppContainer {
    config: Config,
    pool: PgPool,
    clients: Arc<PgClientRepository>,
    cars: Arc<PgCarRepository>,
    bookings: Arc<PgBookingRepository>,
    parts: Arc<PgPartRepository>,
    repairs: Arc<PgRepairRepository>,
    repair_parts: Arc<PgRepairPartRepository>,
    payments: Arc<PgPaymentRepository>,
    stock_movements: Arc<PgStockMovementRepository>,
    client_service: Arc<ClientService<PgClientRepository>>,
    car_service: Arc<CarService<PgClientRepository, PgCarRepository>>,
    booking_service: Arc<BookingService<PgClientRepository, PgCarRepository, PgBookingRepository>>,
    part_service: Arc<PartService<PgPartRepository>>,
    repair_service: Arc<
        RepairService<PgClientRepository, PgCarRepository, PgBookingRepository, PgRepairRepository>,
    >,
    repair_query_service: Arc<
        RepairQueryService<
            PgClientRepository,
            PgCarRepository,
            PgRepairRepository,
            PgRepairPartRepository,
            PgPaymentRepository,
        >,
    >,
}

impl AppContainer {
    pub async fn new(config: Config) -> Result<Self, sqlx::Error> {
        let pool = create_pool(&config.database_url).await?;
        let clients = Arc::new(PgClientRepository::new(pool.clone()));
        let cars = Arc::new(PgCarRepository::new(pool.clone()));
        let bookings = Arc::new(PgBookingRepository::new(pool.clone()));
        let parts = Arc::new(PgPartRepository::new(pool.clone()));
        let repairs = Arc::new(PgRepairRepository::new(pool.clone()));
        let repair_parts = Arc::new(PgRepairPartRepository::new(pool.clone()));
        let payments = Arc::new(PgPaymentRepository::new(pool.clone()));
        let stock_movements = Arc::new(PgStockMovementRepository::new(pool.clone()));
        let client_service = Arc::new(ClientService::new((*clients).clone()));
        let car_service = Arc::new(CarService::new((*clients).clone(), (*cars).clone()));
        let booking_service = Arc::new(BookingService::new(
            (*clients).clone(),
            (*cars).clone(),
            (*bookings).clone(),
        ));
        let part_service = Arc::new(PartService::new((*parts).clone()));
        let repair_service = Arc::new(RepairService::new(
            (*clients).clone(),
            (*cars).clone(),
            (*bookings).clone(),
            (*repairs).clone(),
        ));
        let repair_query_service = Arc::new(RepairQueryService::new(
            (*clients).clone(),
            (*cars).clone(),
            (*repairs).clone(),
            (*repair_parts).clone(),
            (*payments).clone(),
        ));

        Ok(Self {
            config,
            pool,
            clients,
            cars,
            bookings,
            parts,
            repairs,
            repair_parts,
            payments,
            stock_movements,
            client_service,
            car_service,
            booking_service,
            part_service,
            repair_service,
            repair_query_service,
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

    pub fn parts(&self) -> Arc<PgPartRepository> {
        self.parts.clone()
    }

    pub fn repairs(&self) -> Arc<PgRepairRepository> {
        self.repairs.clone()
    }

    pub fn repair_parts(&self) -> Arc<PgRepairPartRepository> {
        self.repair_parts.clone()
    }

    pub fn payments(&self) -> Arc<PgPaymentRepository> {
        self.payments.clone()
    }

    pub fn stock_movements(&self) -> Arc<PgStockMovementRepository> {
        self.stock_movements.clone()
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

    pub fn part_service(&self) -> Arc<PartService<PgPartRepository>> {
        self.part_service.clone()
    }

    pub fn repair_service(
        &self,
    ) -> Arc<
        RepairService<PgClientRepository, PgCarRepository, PgBookingRepository, PgRepairRepository>,
    > {
        self.repair_service.clone()
    }

    pub fn repair_query_service(
        &self,
    ) -> Arc<
        RepairQueryService<
            PgClientRepository,
            PgCarRepository,
            PgRepairRepository,
            PgRepairPartRepository,
            PgPaymentRepository,
        >,
    > {
        self.repair_query_service.clone()
    }

    pub fn timezone_offset_hours(&self) -> i32 {
        self.config.timezone_offset_hours
    }

    pub fn owner_chat_id(&self) -> Option<i64> {
        self.config.owner_chat_id
    }

    pub async fn record_payment(
        &self,
        command: RecordPaymentCommand,
    ) -> AppResult<garage_domain::Payment> {
        let uow = PgPaymentUnitOfWork::begin(&self.pool).await?;
        PaymentTransactionalService::new(uow)
            .record_payment(command)
            .await
    }

    pub async fn use_part_in_repair(
        &self,
        command: UsePartInRepairCommand,
    ) -> AppResult<UsePartInRepairResult> {
        let uow = PgRepairPartUnitOfWork::begin(&self.pool).await?;
        RepairPartTransactionalService::new(uow)
            .use_part_in_repair(command)
            .await
    }
}
