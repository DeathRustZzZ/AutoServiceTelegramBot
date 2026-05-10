use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use garage_domain::*;
use uuid::Uuid;

use crate::*;

#[derive(Default)]
struct Store {
    clients: Mutex<HashMap<ClientId, Client>>,
    cars: Mutex<HashMap<CarId, Car>>,
    bookings: Mutex<HashMap<BookingId, Booking>>,
    parts: Mutex<HashMap<PartId, Part>>,
    supplies: Mutex<HashMap<PartSupplyId, PartSupply>>,
    repairs: Mutex<HashMap<RepairId, Repair>>,
}

#[async_trait]
impl ClientRepository for Store {
    async fn get(&self, id: ClientId) -> AppResult<Option<Client>> {
        Ok(self.clients.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, client: &Client) -> AppResult<()> {
        self.clients
            .lock()
            .unwrap()
            .insert(client.id(), client.clone());
        Ok(())
    }
}

#[async_trait]
impl CarRepository for Store {
    async fn get(&self, id: CarId) -> AppResult<Option<Car>> {
        Ok(self.cars.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, car: &Car) -> AppResult<()> {
        self.cars.lock().unwrap().insert(car.id(), car.clone());
        Ok(())
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Car>> {
        Ok(self
            .cars
            .lock()
            .unwrap()
            .values()
            .filter(|car| car.client_id() == client_id)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl BookingRepository for Store {
    async fn get(&self, id: BookingId) -> AppResult<Option<Booking>> {
        Ok(self.bookings.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, booking: &Booking) -> AppResult<()> {
        self.bookings
            .lock()
            .unwrap()
            .insert(booking.id(), booking.clone());
        Ok(())
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Booking>> {
        Ok(self
            .bookings
            .lock()
            .unwrap()
            .values()
            .filter(|booking| booking.client_id() == client_id)
            .cloned()
            .collect())
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Booking>> {
        Ok(self
            .bookings
            .lock()
            .unwrap()
            .values()
            .filter(|booking| booking.car_id() == car_id)
            .cloned()
            .collect())
    }

    async fn list_scheduled_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Booking>> {
        Ok(self
            .bookings
            .lock()
            .unwrap()
            .values()
            .filter(|booking| {
                booking.is_scheduled()
                    && *booking.scheduled_at() >= from
                    && *booking.scheduled_at() < to
            })
            .cloned()
            .collect())
    }
}

#[async_trait]
impl PartRepository for Store {
    async fn get(&self, id: PartId) -> AppResult<Option<Part>> {
        Ok(self.parts.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, part: &Part) -> AppResult<()> {
        self.parts.lock().unwrap().insert(part.id(), part.clone());
        Ok(())
    }

    async fn list_low_stock(&self) -> AppResult<Vec<Part>> {
        Ok(self
            .parts
            .lock()
            .unwrap()
            .values()
            .filter(|part| part.is_low_stock())
            .cloned()
            .collect())
    }

    async fn search(&self, query: &str) -> AppResult<Vec<Part>> {
        let query = query.trim().to_lowercase();

        Ok(self
            .parts
            .lock()
            .unwrap()
            .values()
            .filter(|part| {
                part.name().as_str().to_lowercase().contains(&query)
                    || part
                        .sku()
                        .is_some_and(|sku| sku.as_str().to_lowercase().contains(&query))
            })
            .cloned()
            .collect())
    }
}

#[async_trait]
impl PartSupplyRepository for Store {
    async fn get(&self, id: PartSupplyId) -> AppResult<Option<PartSupply>> {
        Ok(self.supplies.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, supply: &PartSupply) -> AppResult<()> {
        self.supplies
            .lock()
            .unwrap()
            .insert(supply.id(), supply.clone());
        Ok(())
    }

    async fn list_by_part(&self, part_id: PartId) -> AppResult<Vec<PartSupply>> {
        Ok(self
            .supplies
            .lock()
            .unwrap()
            .values()
            .filter(|supply| supply.part_id() == part_id)
            .cloned()
            .collect())
    }
}

#[async_trait]
impl RepairRepository for Store {
    async fn get(&self, id: RepairId) -> AppResult<Option<Repair>> {
        Ok(self.repairs.lock().unwrap().get(&id).cloned())
    }

    async fn save(&self, repair: &Repair) -> AppResult<()> {
        self.repairs
            .lock()
            .unwrap()
            .insert(repair.id(), repair.clone());
        Ok(())
    }

    async fn list_by_client(&self, client_id: ClientId) -> AppResult<Vec<Repair>> {
        Ok(self
            .repairs
            .lock()
            .unwrap()
            .values()
            .filter(|repair| repair.client_id() == client_id)
            .cloned()
            .collect())
    }

    async fn list_by_car(&self, car_id: CarId) -> AppResult<Vec<Repair>> {
        Ok(self
            .repairs
            .lock()
            .unwrap()
            .values()
            .filter(|repair| repair.car_id() == car_id)
            .cloned()
            .collect())
    }

    async fn list_completed_between(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> AppResult<Vec<Repair>> {
        Ok(self
            .repairs
            .lock()
            .unwrap()
            .values()
            .filter(|repair| {
                repair
                    .completed_at()
                    .is_some_and(|completed_at| *completed_at >= from && *completed_at <= to)
            })
            .cloned()
            .collect())
    }
}

fn store() -> Arc<Store> {
    Arc::new(Store::default())
}

fn ts(hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 10, hour, 0, 0).unwrap()
}

fn client_name(value: &str) -> ClientName {
    ClientName::parse(value).unwrap()
}

fn phone(value: &str) -> PhoneNumber {
    PhoneNumber::parse(value).unwrap()
}

fn car_make(value: &str) -> CarMake {
    CarMake::parse(value).unwrap()
}

fn car_model(value: &str) -> CarModel {
    CarModel::parse(value).unwrap()
}

fn reason(value: &str) -> BookingReason {
    BookingReason::parse(value).unwrap()
}

fn part_name(value: &str) -> PartName {
    PartName::parse(value).unwrap()
}

fn sku(value: &str) -> Option<PartSku> {
    PartSku::parse(value).unwrap()
}

fn description(value: &str) -> RepairDescription {
    RepairDescription::parse(value).unwrap()
}

fn document_photo(value: &str) -> CarDocumentPhotoRef {
    CarDocumentPhotoRef::new(value).unwrap()
}

fn start_repair_command(
    client_id: ClientId,
    car_id: CarId,
    booking_id: Option<BookingId>,
) -> StartRepairCommand {
    StartRepairCommand {
        client_id,
        car_id,
        booking_id,
        description: description("Ремонт"),
        labor_price: Money::byn_minor(1000).unwrap(),
        parts_price: Money::byn_minor(0).unwrap(),
        parts_cost: Money::byn_minor(0).unwrap(),
        notes: None,
        now: ts(9),
    }
}

async fn create_client_fixture(store: Arc<Store>, name: &str, phone: &str) -> Client {
    ClientService::new(store)
        .create_client(client_name(name), self::phone(phone), None, ts(8))
        .await
        .unwrap()
}

async fn create_car_fixture(
    store: Arc<Store>,
    client_id: ClientId,
    make: &str,
    model: &str,
) -> Car {
    CarService::new(store.clone(), store)
        .create_car(
            client_id,
            car_make(make),
            car_model(model),
            Some(CarYear::new(2018).unwrap()),
            None,
            None,
            None,
            ts(9),
        )
        .await
        .unwrap()
}

async fn create_part_fixture(store: Arc<Store>, name: &str, sku: &str, quantity: u32) -> Part {
    PartService::new(store)
        .create_part(
            part_name(name),
            self::sku(sku),
            PartQuantity::new(quantity),
            PartQuantity::new(2),
            Money::byn_minor(1000).unwrap(),
            None,
            ts(8),
        )
        .await
        .unwrap()
}

#[tokio::test]
async fn client_service_creates_and_updates_client() {
    let store = store();
    let service = ClientService::new(store.clone());

    let client = service
        .create_client(
            client_name("Иван"),
            phone("+375291111111"),
            ClientNotes::parse("VIP").unwrap(),
            ts(8),
        )
        .await
        .unwrap();

    let client = service
        .rename_client(client.id(), client_name("Петр"), ts(9))
        .await
        .unwrap();
    assert_eq!(client.name().as_str(), "Петр");

    let client = service
        .change_phone(client.id(), phone("80292222222"), ts(10))
        .await
        .unwrap();
    assert_eq!(client.phone().as_str(), "+375292222222");

    let client = service
        .update_notes(client.id(), None, ts(11))
        .await
        .unwrap();
    assert!(client.notes().is_none());
    assert_eq!(
        ClientRepository::get(&store, client.id())
            .await
            .unwrap()
            .unwrap(),
        client
    );
}

#[tokio::test]
async fn car_service_checks_client_and_lists_client_cars() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car_service = CarService::new(store.clone(), store.clone());

    let car = car_service
        .create_car(
            client.id(),
            car_make("BMW"),
            car_model("X5"),
            None,
            LicensePlate::parse("1234 ab-7").unwrap(),
            None,
            None,
            ts(9),
        )
        .await
        .unwrap();

    let cars = car_service.list_client_cars(client.id()).await.unwrap();
    assert_eq!(cars, vec![car.clone()]);

    let updated = car_service
        .update_identity(
            car.id(),
            car_make("Audi"),
            car_model("A6"),
            Some(CarYear::new(2020).unwrap()),
            None,
            None,
            ts(10),
        )
        .await
        .unwrap();
    assert_eq!(updated.make().as_str(), "Audi");

    let missing_client = ClientId::from_uuid(Uuid::from_u128(1));
    let result = car_service.list_client_cars(missing_client).await;
    assert!(matches!(result, Err(AppError::ClientNotFound(id)) if id == missing_client));
}

#[tokio::test]
async fn set_registration_document_photo_fails_when_client_not_found() {
    let store = store();
    let existing_client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), existing_client.id(), "BMW", "X5").await;
    let missing_client = ClientId::from_uuid(Uuid::from_u128(10));
    let service = CarService::new(store.clone(), store);

    let result = service
        .set_registration_document_photo(
            missing_client,
            car.id(),
            document_photo("telegram-file-id"),
            ts(10),
        )
        .await;

    assert!(matches!(result, Err(AppError::ClientNotFound(id)) if id == missing_client));
}

#[tokio::test]
async fn set_registration_document_photo_fails_when_car_not_found() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let missing_car = CarId::from_uuid(Uuid::from_u128(11));
    let service = CarService::new(store.clone(), store);

    let result = service
        .set_registration_document_photo(
            client.id(),
            missing_car,
            document_photo("telegram-file-id"),
            ts(10),
        )
        .await;

    assert!(matches!(result, Err(AppError::CarNotFound(id)) if id == missing_car));
}

#[tokio::test]
async fn set_registration_document_photo_fails_when_car_does_not_belong_to_client() {
    let store = store();
    let first = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let second = create_client_fixture(store.clone(), "Петр", "+375292222222").await;
    let car = create_car_fixture(store.clone(), first.id(), "BMW", "X5").await;
    let service = CarService::new(store.clone(), store);

    let result = service
        .set_registration_document_photo(
            second.id(),
            car.id(),
            document_photo("telegram-file-id"),
            ts(10),
        )
        .await;

    assert!(matches!(
        result,
        Err(AppError::CarDoesNotBelongToClient { car_id, client_id })
            if car_id == car.id() && client_id == second.id()
    ));
}

#[tokio::test]
async fn set_registration_document_photo_saves_updated_car() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = CarService::new(store.clone(), store.clone());

    let updated = service
        .set_registration_document_photo(
            client.id(),
            car.id(),
            document_photo("telegram-file-id"),
            ts(10),
        )
        .await
        .unwrap();

    assert_eq!(
        updated.registration_document_photo().unwrap().as_str(),
        "telegram-file-id"
    );
    assert_eq!(*updated.updated_at(), ts(10));
    assert_eq!(
        CarRepository::get(&store, car.id())
            .await
            .unwrap()
            .unwrap()
            .registration_document_photo()
            .unwrap()
            .as_str(),
        "telegram-file-id"
    );
}

#[tokio::test]
async fn remove_registration_document_photo_saves_updated_car() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = CarService::new(store.clone(), store.clone());

    service
        .set_registration_document_photo(
            client.id(),
            car.id(),
            document_photo("telegram-file-id"),
            ts(10),
        )
        .await
        .unwrap();

    let updated = service
        .remove_registration_document_photo(client.id(), car.id(), ts(11))
        .await
        .unwrap();

    assert!(updated.registration_document_photo().is_none());
    assert_eq!(*updated.updated_at(), ts(11));
    assert!(CarRepository::get(&store, car.id())
        .await
        .unwrap()
        .unwrap()
        .registration_document_photo()
        .is_none());
}

#[tokio::test]
async fn remove_registration_document_photo_fails_when_car_does_not_belong_to_client() {
    let store = store();
    let first = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let second = create_client_fixture(store.clone(), "Петр", "+375292222222").await;
    let car = create_car_fixture(store.clone(), first.id(), "BMW", "X5").await;
    let service = CarService::new(store.clone(), store);

    let result = service
        .remove_registration_document_photo(second.id(), car.id(), ts(11))
        .await;

    assert!(matches!(
        result,
        Err(AppError::CarDoesNotBelongToClient { car_id, client_id })
            if car_id == car.id() && client_id == second.id()
    ));
}

#[tokio::test]
async fn booking_service_schedules_lists_and_transitions_bookings() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = BookingService::new(store.clone(), store.clone(), store.clone());

    let booking = service
        .schedule_booking(
            client.id(),
            car.id(),
            ts(12),
            reason("Диагностика"),
            None,
            ts(8),
        )
        .await
        .unwrap();
    assert!(booking.is_scheduled());

    assert_eq!(
        service
            .list_client_bookings(client.id())
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(service.list_car_bookings(car.id()).await.unwrap().len(), 1);
    assert_eq!(
        service
            .list_bookings_between(ts(11), ts(13))
            .await
            .unwrap()
            .len(),
        1
    );

    let booking = service
        .reschedule_booking(booking.id(), ts(14), ts(9))
        .await
        .unwrap();
    assert_eq!(*booking.scheduled_at(), ts(14));

    let booking = service
        .complete_booking(booking.id(), ts(15))
        .await
        .unwrap();
    assert_eq!(booking.status(), BookingStatus::Completed);
    assert!(service
        .list_bookings_between(ts(13), ts(15))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn schedule_booking_fails_when_car_does_not_belong_to_client() {
    let store = store();
    let first = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let second = create_client_fixture(store.clone(), "Петр", "+375292222222").await;
    let car = create_car_fixture(store.clone(), first.id(), "BMW", "X5").await;
    let service = BookingService::new(store.clone(), store.clone(), store.clone());

    let result = service
        .schedule_booking(
            second.id(),
            car.id(),
            ts(12),
            reason("Диагностика"),
            None,
            ts(8),
        )
        .await;

    assert!(matches!(
        result,
        Err(AppError::CarDoesNotBelongToClient { car_id, client_id })
            if car_id == car.id() && client_id == second.id()
    ));
}

#[tokio::test]
async fn get_booking_details_returns_booking_client_and_car() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = BookingService::new(store.clone(), store.clone(), store.clone());
    let booking = service
        .schedule_booking(
            client.id(),
            car.id(),
            ts(12),
            reason("Диагностика"),
            None,
            ts(8),
        )
        .await
        .unwrap();

    let details = service.get_booking_details(booking.id()).await.unwrap();
    assert_eq!(details.booking, booking);
    assert_eq!(details.client, client);
    assert_eq!(details.car, car);
}

#[tokio::test]
async fn list_booking_details_between_returns_details_for_scheduled_bookings() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = BookingService::new(store.clone(), store.clone(), store.clone());
    let booking = service
        .schedule_booking(
            client.id(),
            car.id(),
            ts(12),
            reason("Диагностика"),
            None,
            ts(8),
        )
        .await
        .unwrap();
    service
        .schedule_booking(client.id(), car.id(), ts(18), reason("Позже"), None, ts(8))
        .await
        .unwrap();

    let details = service
        .list_booking_details_between(ts(11), ts(13))
        .await
        .unwrap();

    assert_eq!(details.len(), 1);
    assert_eq!(details[0].booking, booking);
    assert_eq!(details[0].client, client);
    assert_eq!(details[0].car, car);
}

#[tokio::test]
async fn list_today_bookings_uses_current_day_boundaries() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = BookingService::new(store.clone(), store.clone(), store.clone());
    let today = service
        .schedule_booking(
            client.id(),
            car.id(),
            Utc.with_ymd_and_hms(2026, 5, 10, 23, 59, 0).unwrap(),
            reason("Сегодня"),
            None,
            ts(8),
        )
        .await
        .unwrap();
    service
        .schedule_booking(
            client.id(),
            car.id(),
            Utc.with_ymd_and_hms(2026, 5, 11, 0, 0, 0).unwrap(),
            reason("Завтра"),
            None,
            ts(8),
        )
        .await
        .unwrap();

    let bookings = service
        .list_today_bookings(Utc.with_ymd_and_hms(2026, 5, 10, 15, 30, 0).unwrap())
        .await
        .unwrap();
    assert_eq!(bookings, vec![today]);
}

#[tokio::test]
async fn list_tomorrow_bookings_uses_next_day_boundaries() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = BookingService::new(store.clone(), store.clone(), store.clone());
    service
        .schedule_booking(
            client.id(),
            car.id(),
            Utc.with_ymd_and_hms(2026, 5, 10, 23, 59, 0).unwrap(),
            reason("Сегодня"),
            None,
            ts(8),
        )
        .await
        .unwrap();
    let tomorrow = service
        .schedule_booking(
            client.id(),
            car.id(),
            Utc.with_ymd_and_hms(2026, 5, 11, 0, 0, 0).unwrap(),
            reason("Завтра"),
            None,
            ts(8),
        )
        .await
        .unwrap();
    service
        .schedule_booking(
            client.id(),
            car.id(),
            Utc.with_ymd_and_hms(2026, 5, 12, 0, 0, 0).unwrap(),
            reason("Послезавтра"),
            None,
            ts(8),
        )
        .await
        .unwrap();

    let bookings = service
        .list_tomorrow_bookings(Utc.with_ymd_and_hms(2026, 5, 10, 15, 30, 0).unwrap())
        .await
        .unwrap();
    assert_eq!(bookings, vec![tomorrow]);
}

#[tokio::test]
async fn part_service_creates_sets_stock_searches_and_lists_low_stock() {
    let store = store();
    let service = PartService::new(store.clone());

    let oil = service
        .create_part(
            part_name("Масляный фильтр"),
            sku("oil-001"),
            PartQuantity::new(1),
            PartQuantity::new(2),
            Money::byn_minor(2500).unwrap(),
            None,
            ts(8),
        )
        .await
        .unwrap();
    create_part_fixture(store.clone(), "Воздушный фильтр", "air-001", 10).await;

    assert_eq!(
        service.search_parts("oil").await.unwrap(),
        vec![oil.clone()]
    );
    assert_eq!(service.list_low_stock().await.unwrap(), vec![oil.clone()]);

    let updated = service
        .set_stock(oil.id(), PartQuantity::new(5), ts(9))
        .await
        .unwrap();
    assert_eq!(updated.quantity().value(), 5);
    assert!(service.list_low_stock().await.unwrap().is_empty());
}

#[tokio::test]
async fn part_supply_service_receives_supply_and_updates_stock() {
    let store = store();
    let part = create_part_fixture(store.clone(), "Фильтр", "flt-001", 1).await;
    let service = PartSupplyService::new(store.clone(), store.clone());

    let supply = service
        .create_supply(
            part.id(),
            PartQuantity::new(4),
            ts(12),
            PartSupplier::parse("Поставщик").unwrap(),
            None,
            ts(8),
        )
        .await
        .unwrap();

    let (supply, part) = service.receive_supply(supply.id(), ts(13)).await.unwrap();
    assert_eq!(supply.status(), PartSupplyStatus::Received);
    assert_eq!(part.quantity().value(), 5);

    let result = service.cancel_supply(supply.id(), ts(14)).await;
    assert!(matches!(result, Err(AppError::PartSupply(_))));
}

#[tokio::test]
async fn repair_service_starts_records_payment_and_completes_repair() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let booking = BookingService::new(store.clone(), store.clone(), store.clone())
        .schedule_booking(client.id(), car.id(), ts(12), reason("Ремонт"), None, ts(8))
        .await
        .unwrap();
    let service = RepairService::new(store.clone(), store.clone(), store.clone(), store.clone());

    let repair = service
        .start_repair(StartRepairCommand {
            client_id: client.id(),
            car_id: car.id(),
            booking_id: Some(booking.id()),
            description: description("Замена масла"),
            labor_price: Money::byn_minor(5000).unwrap(),
            parts_price: Money::byn_minor(3000).unwrap(),
            parts_cost: Money::byn_minor(2000).unwrap(),
            notes: None,
            now: ts(9),
        })
        .await
        .unwrap();

    let repair = service
        .record_payment(repair.id(), Money::byn_minor(8000).unwrap(), ts(10))
        .await
        .unwrap();
    assert_eq!(repair.payment_status().unwrap(), PaymentStatus::Paid);

    let repair = service.complete_repair(repair.id(), ts(11)).await.unwrap();
    assert_eq!(repair.status(), RepairStatus::Completed);
    assert_eq!(repair.actual_profit().unwrap().amount_minor(), 6000);
}

#[tokio::test]
async fn start_repair_fails_when_car_does_not_belong_to_client() {
    let store = store();
    let first = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let second = create_client_fixture(store.clone(), "Петр", "+375292222222").await;
    let car = create_car_fixture(store.clone(), first.id(), "BMW", "X5").await;
    let service = RepairService::new(store.clone(), store.clone(), store.clone(), store.clone());

    let result = service
        .start_repair(start_repair_command(second.id(), car.id(), None))
        .await;

    assert!(matches!(
        result,
        Err(AppError::CarDoesNotBelongToClient { car_id, client_id })
            if car_id == car.id() && client_id == second.id()
    ));
}

#[tokio::test]
async fn start_repair_fails_when_booking_belongs_to_another_car() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let first_car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let second_car = create_car_fixture(store.clone(), client.id(), "Audi", "A6").await;
    let booking = BookingService::new(store.clone(), store.clone(), store.clone())
        .schedule_booking(
            client.id(),
            first_car.id(),
            ts(12),
            reason("Диагностика"),
            None,
            ts(8),
        )
        .await
        .unwrap();
    let service = RepairService::new(store.clone(), store.clone(), store.clone(), store.clone());

    let result = service
        .start_repair(start_repair_command(
            client.id(),
            second_car.id(),
            Some(booking.id()),
        ))
        .await;

    assert!(matches!(
        result,
        Err(AppError::BookingDoesNotBelongToCar { booking_id, car_id })
            if booking_id == booking.id() && car_id == second_car.id()
    ));
}

#[tokio::test]
async fn repair_service_cancels_repair_and_rejects_later_payment() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let service = RepairService::new(store.clone(), store.clone(), store.clone(), store.clone());

    let repair = service
        .start_repair(start_repair_command(client.id(), car.id(), None))
        .await
        .unwrap();

    let repair = service.cancel_repair(repair.id(), ts(10)).await.unwrap();
    assert_eq!(repair.status(), RepairStatus::Cancelled);

    let result = service
        .record_payment(repair.id(), Money::byn_minor(1000).unwrap(), ts(11))
        .await;
    assert!(matches!(result, Err(AppError::Repair(_))));
}

#[tokio::test]
async fn statistics_service_calculates_profit_summary_for_currency() {
    let store = store();
    let client = create_client_fixture(store.clone(), "Иван", "+375291111111").await;
    let car = create_car_fixture(store.clone(), client.id(), "BMW", "X5").await;
    let repair_service =
        RepairService::new(store.clone(), store.clone(), store.clone(), store.clone());

    let byn_repair = repair_service
        .start_repair(StartRepairCommand {
            client_id: client.id(),
            car_id: car.id(),
            booking_id: None,
            description: description("BYN repair"),
            labor_price: Money::byn_minor(5000).unwrap(),
            parts_price: Money::byn_minor(3000).unwrap(),
            parts_cost: Money::byn_minor(2000).unwrap(),
            notes: None,
            now: ts(9),
        })
        .await
        .unwrap();
    repair_service
        .record_payment(byn_repair.id(), Money::byn_minor(7000).unwrap(), ts(10))
        .await
        .unwrap();
    repair_service
        .complete_repair(byn_repair.id(), ts(11))
        .await
        .unwrap();

    let usd_repair = repair_service
        .start_repair(StartRepairCommand {
            client_id: client.id(),
            car_id: car.id(),
            booking_id: None,
            description: description("USD repair"),
            labor_price: Money::usd_minor(1000).unwrap(),
            parts_price: Money::usd_minor(0).unwrap(),
            parts_cost: Money::usd_minor(0).unwrap(),
            notes: None,
            now: ts(9),
        })
        .await
        .unwrap();
    repair_service
        .complete_repair(usd_repair.id(), ts(11))
        .await
        .unwrap();

    let summary = StatisticsService::new(store)
        .profit_summary(ts(8), ts(12), Currency::Byn)
        .await
        .unwrap();

    assert_eq!(summary.completed_repairs, 1);
    assert_eq!(summary.revenue.amount_minor(), 8000);
    assert_eq!(summary.parts_cost.amount_minor(), 2000);
    assert_eq!(summary.expected_profit.amount_minor(), 6000);
    assert_eq!(summary.actual_profit.amount_minor(), 5000);
}
