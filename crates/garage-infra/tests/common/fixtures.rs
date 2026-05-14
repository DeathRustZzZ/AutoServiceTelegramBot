use chrono::{DateTime, Duration, TimeZone, Utc};
use garage_domain::{
    Car, CarId, CarMake, CarModel, Client, ClientId, ClientName, Money, Part, PartId, PartName,
    PartQuantity, PartSku, Payment, PaymentId, PaymentMethod, PhoneNumber, Repair,
    RepairDescription, RepairId,
};
use uuid::Uuid;

pub fn fixed_time(seconds: i64) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 14, 14, 0, 0)
        .single()
        .expect("valid fixed timestamp")
        + Duration::seconds(seconds)
}

pub fn client(id: u128) -> Client {
    Client::new(
        ClientId::from_uuid(Uuid::from_u128(id)),
        ClientName::parse("Иван Петров").expect("valid client name"),
        PhoneNumber::parse("+375291234567").expect("valid phone number"),
        None,
        fixed_time(0),
    )
}

pub fn car(id: u128, client_id: ClientId) -> Car {
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

pub fn part(id: u128, name: &str, sku: Option<&str>, quantity: u32, min_quantity: u32) -> Part {
    Part::new(
        PartId::from_uuid(Uuid::from_u128(id)),
        PartName::parse(name).expect("valid part name"),
        sku.map(PartSku::parse)
            .transpose()
            .expect("valid part sku")
            .flatten(),
        PartQuantity::new(quantity),
        PartQuantity::new(min_quantity),
        Money::byn_minor(2_500).expect("valid part unit price"),
        None,
        fixed_time(2),
    )
}

pub fn repair(id: u128, client_id: ClientId, car_id: CarId) -> Repair {
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
        fixed_time(3),
    )
    .expect("valid repair")
}

pub fn payment(id: u128, repair_id: RepairId, amount_minor: i64) -> Payment {
    Payment::new(
        PaymentId::from_uuid(Uuid::from_u128(id)),
        repair_id,
        Money::byn_minor(amount_minor).expect("valid payment amount"),
        PaymentMethod::Cash,
        None,
        fixed_time(4),
        fixed_time(5),
    )
    .expect("valid payment")
}
