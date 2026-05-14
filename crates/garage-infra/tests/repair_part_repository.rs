mod common;

use common::fixtures;
use garage_app::{
    CarRepository, ClientRepository, PartRepository, RepairPartRepository, RepairRepository,
};
use garage_domain::{Money, PartQuantity, RepairPart, RepairPartId};
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use garage_infra::repositories::part::PgPartRepository;
use garage_infra::repositories::repair::PgRepairRepository;
use garage_infra::repositories::repair_part::PgRepairPartRepository;
use uuid::Uuid;

#[tokio::test]
async fn repair_part_repository_saves_loads_and_lists_by_repair() {
    let db = common::setup_test_db().await;
    let clients = PgClientRepository::new(db.pool());
    let cars = PgCarRepository::new(db.pool());
    let repairs = PgRepairRepository::new(db.pool());
    let parts = PgPartRepository::new(db.pool());
    let repair_parts = PgRepairPartRepository::new(db.pool());
    let client = fixtures::client(1);
    let car = fixtures::car(2, client.id());
    let repair = fixtures::repair(3, client.id(), car.id());
    let part = fixtures::part(4, "Oil filter", Some("of-123"), 10, 3);

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");
    repairs.save(&repair).await.expect("repair should be saved");
    parts.save(&part).await.expect("part should be saved");

    let repair_part = RepairPart::new(
        RepairPartId::from_uuid(Uuid::from_u128(5)),
        repair.id(),
        part.id(),
        PartQuantity::new(2),
        Money::byn_minor(1_000).expect("valid unit cost"),
        Money::byn_minor(2_500).expect("valid unit price"),
        fixtures::fixed_time(10),
    )
    .expect("valid repair part");

    repair_parts
        .save(&repair_part)
        .await
        .expect("repair part should be saved");

    let loaded = repair_parts
        .get(repair_part.id())
        .await
        .expect("repair part should be loaded")
        .expect("repair part should exist");
    assert_eq!(loaded, repair_part);

    let listed = repair_parts
        .list_by_repair(repair.id())
        .await
        .expect("repair parts should be listed by repair");
    assert!(listed.contains(&repair_part));

    let part_after_repair_part = parts
        .get(part.id())
        .await
        .expect("part should be loaded after repair part")
        .expect("part should still exist");
    assert_eq!(part_after_repair_part.quantity(), part.quantity());
}
