mod common;

use chrono::{DateTime, Duration, TimeZone, Utc};
use garage_app::{CarRepository, ClientRepository, RepairRepository};
use garage_domain::{
    Car, CarId, CarMake, CarModel, Client, ClientId, ClientName, Money, PhoneNumber, Repair,
    RepairDescription, RepairId,
};
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use garage_infra::repositories::repair::PgRepairRepository;
use uuid::Uuid;

fn fixed_time(seconds: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 14, 11, 0, 0)
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

fn repair(id: u128, client_id: ClientId, car_id: CarId, now: DateTime<Utc>) -> Repair {
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
        now,
    )
    .expect("valid repair")
}

#[tokio::test]
async fn repair_repository_saves_loads_lists_and_filters_completed_between() {
    let db = common::setup_test_db().await;
    let clients = PgClientRepository::new(db.pool());
    let cars = PgCarRepository::new(db.pool());
    let repairs = PgRepairRepository::new(db.pool());
    let client = client(1);
    let car = car(2, client.id());

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");

    let in_progress = repair(3, client.id(), car.id(), fixed_time(2));
    repairs
        .save(&in_progress)
        .await
        .expect("in-progress repair should be saved");

    let loaded = repairs
        .get(in_progress.id())
        .await
        .expect("repair should be loaded")
        .expect("repair should exist");
    assert_eq!(loaded, in_progress);

    let by_client = repairs
        .list_by_client(client.id())
        .await
        .expect("repairs should be listed by client");
    assert!(by_client.contains(&in_progress));

    let by_car = repairs
        .list_by_car(car.id())
        .await
        .expect("repairs should be listed by car");
    assert!(by_car.contains(&in_progress));

    let mut completed = repair(4, client.id(), car.id(), fixed_time(10));
    let completed_at = fixed_time(3_600);
    completed
        .complete(completed_at)
        .expect("repair should be completed");
    repairs
        .save(&completed)
        .await
        .expect("completed repair should be saved");

    let completed_between = repairs
        .list_completed_between(
            completed_at - Duration::minutes(30),
            completed_at + Duration::minutes(30),
        )
        .await
        .expect("completed repairs should be listed by range");
    assert!(completed_between.contains(&completed));
    assert!(!completed_between.contains(&in_progress));

    let outside = repairs
        .list_completed_between(
            completed_at + Duration::hours(2),
            completed_at + Duration::hours(3),
        )
        .await
        .expect("completed repairs outside range should be listed");
    assert!(!outside.contains(&completed));
}
