mod common;

use chrono::{DateTime, Duration, TimeZone, Utc};
use garage_app::{BookingRepository, CarRepository, ClientRepository};
use garage_domain::{
    Booking, BookingId, BookingReason, Car, CarId, CarMake, CarModel, Client, ClientId, ClientName,
    PhoneNumber,
};
use garage_infra::repositories::booking::PgBookingRepository;
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use uuid::Uuid;

fn fixed_time(seconds: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 14, 9, 0, 0)
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

#[tokio::test]
async fn booking_repository_saves_loads_lists_by_client_car_and_scheduled_range() {
    let db = common::setup_test_db().await;
    let clients = PgClientRepository::new(db.pool());
    let cars = PgCarRepository::new(db.pool());
    let bookings = PgBookingRepository::new(db.pool());
    let client = client(1);
    let car = car(2, client.id());

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");

    let scheduled_at = fixed_time(3_600);
    let booking = Booking::new(
        BookingId::from_uuid(Uuid::from_u128(3)),
        client.id(),
        car.id(),
        scheduled_at,
        BookingReason::parse("Диагностика").expect("valid booking reason"),
        None,
        fixed_time(2),
    );

    bookings
        .save(&booking)
        .await
        .expect("booking should be saved");

    let loaded = bookings
        .get(booking.id())
        .await
        .expect("booking should be loaded")
        .expect("booking should exist");
    assert_eq!(loaded, booking);

    let by_client = bookings
        .list_by_client(client.id())
        .await
        .expect("bookings should be listed by client");
    assert!(by_client.contains(&booking));

    let by_car = bookings
        .list_by_car(car.id())
        .await
        .expect("bookings should be listed by car");
    assert!(by_car.contains(&booking));

    let scheduled = bookings
        .list_scheduled_between(
            scheduled_at - Duration::minutes(30),
            scheduled_at + Duration::minutes(30),
        )
        .await
        .expect("scheduled bookings should be listed by range");
    assert!(scheduled.contains(&booking));

    let outside = bookings
        .list_scheduled_between(
            scheduled_at + Duration::hours(2),
            scheduled_at + Duration::hours(3),
        )
        .await
        .expect("scheduled bookings outside range should be listed");
    assert!(!outside.contains(&booking));
}
