mod common;

use chrono::{DateTime, Duration, TimeZone, Utc};
use garage_app::{CarRepository, ClientRepository, PaymentRepository, RepairRepository};
use garage_domain::{
    Car, CarId, CarMake, CarModel, Client, ClientId, ClientName, Money, Payment, PaymentId,
    PaymentMethod, PhoneNumber, Repair, RepairDescription, RepairId,
};
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use garage_infra::repositories::payment::PgPaymentRepository;
use garage_infra::repositories::repair::PgRepairRepository;
use uuid::Uuid;

fn fixed_time(seconds: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 14, 13, 0, 0)
        .single()
        .expect("valid fixed timestamp")
        + Duration::seconds(seconds)
}

fn client(id: u128) -> Client {
    Client::new(
        ClientId::from_uuid(Uuid::from_u128(id)),
        ClientName::parse("Иван Петров").expect("valid client name"),
        PhoneNumber::parse("+375291234567").expect("valid phone number"),
        None,
        fixed_time(0),
    )
}

fn car(id: u128, client_id: ClientId) -> Car {
    Car::new(
        CarId::from_uuid(Uuid::from_u128(id)),
        client_id,
        CarMake::parse("Toyota").expect("valid car make"),
        CarModel::parse("Camry").expect("valid car model"),
        None,
        None,
        None,
        None,
        fixed_time(1),
    )
}

fn repair(id: u128, client_id: ClientId, car_id: CarId) -> Repair {
    Repair::new(
        RepairId::from_uuid(Uuid::from_u128(id)),
        client_id,
        car_id,
        None,
        RepairDescription::parse("Замена масла").expect("valid repair description"),
        Money::byn_minor(10_000).expect("valid labor price"),
        Money::byn_minor(2_000).expect("valid parts price"),
        Money::byn_minor(1_000).expect("valid parts cost"),
        None,
        fixed_time(2),
    )
    .expect("valid repair")
}

#[tokio::test]
async fn payment_repository_saves_loads_and_lists_by_repair() {
    let db = common::setup_test_db().await;
    let clients = PgClientRepository::new(db.pool());
    let cars = PgCarRepository::new(db.pool());
    let repairs = PgRepairRepository::new(db.pool());
    let payments = PgPaymentRepository::new(db.pool());
    let client = client(1);
    let car = car(2, client.id());
    let repair = repair(3, client.id(), car.id());

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");
    repairs.save(&repair).await.expect("repair should be saved");

    let payment = Payment::new(
        PaymentId::from_uuid(Uuid::from_u128(4)),
        repair.id(),
        Money::byn_minor(5_000).expect("valid payment amount"),
        PaymentMethod::Cash,
        None,
        fixed_time(3),
        fixed_time(4),
    )
    .expect("valid payment");

    payments
        .save(&payment)
        .await
        .expect("payment should be saved");

    let loaded = payments
        .get(payment.id())
        .await
        .expect("payment should be loaded")
        .expect("payment should exist");
    assert_eq!(loaded, payment);

    let listed = payments
        .list_by_repair(repair.id())
        .await
        .expect("payments should be listed by repair");
    assert!(listed.contains(&payment));

    let repair_after_payment = repairs
        .get(repair.id())
        .await
        .expect("repair should be loaded after payment")
        .expect("repair should still exist");
    assert_eq!(repair_after_payment.paid_amount(), repair.paid_amount());
}
