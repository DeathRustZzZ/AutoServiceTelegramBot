mod common;

use chrono::{DateTime, TimeZone, Utc};
use garage_app::{CarRepository, ClientRepository};
use garage_domain::{Car, CarId, CarMake, CarModel, Client, ClientId, ClientName, PhoneNumber};
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use uuid::Uuid;

fn fixed_time(seconds: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 14, 9, 0, seconds)
        .single()
        .expect("valid fixed timestamp")
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

#[tokio::test]
async fn car_repository_saves_loads_and_lists_by_client() {
    let db = common::setup_test_db().await;
    let clients = PgClientRepository::new(db.pool());
    let cars = PgCarRepository::new(db.pool());
    let client = client(1);

    clients.save(&client).await.expect("client should be saved");

    let car = Car::new(
        CarId::from_uuid(Uuid::from_u128(2)),
        client.id(),
        CarMake::parse("Toyota").expect("valid car make"),
        CarModel::parse("Camry").expect("valid car model"),
        None,
        None,
        None,
        None,
        fixed_time(1),
    );

    cars.save(&car).await.expect("car should be saved");

    let loaded = cars
        .get(car.id())
        .await
        .expect("car should be loaded")
        .expect("car should exist");
    assert_eq!(loaded, car);

    let listed = cars
        .list_by_client(client.id())
        .await
        .expect("cars should be listed by client");
    assert_eq!(listed, vec![car]);
}
