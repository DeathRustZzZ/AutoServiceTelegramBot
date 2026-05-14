mod common;

use common::fixtures;
use garage_app::{
    CarRepository, ClientRepository, PartRepository, RepairPartRepository, RepairPartUnitOfWork,
    RepairRepository, StockMovementRepository,
};
use garage_domain::{
    Money, PartQuantity, RepairPart, RepairPartId, StockMovement, StockMovementId,
    StockMovementReason, StockMovementType,
};
use garage_infra::repositories::car::PgCarRepository;
use garage_infra::repositories::client::PgClientRepository;
use garage_infra::repositories::part::PgPartRepository;
use garage_infra::repositories::repair::PgRepairRepository;
use garage_infra::repositories::repair_part::PgRepairPartRepository;
use garage_infra::repositories::stock_movement::PgStockMovementRepository;
use garage_infra::unit_of_work::repair_part::PgRepairPartUnitOfWork;
use uuid::Uuid;

#[tokio::test]
async fn repair_part_unit_of_work_commits_part_repair_part_stock_movement_and_repair() {
    let db = common::setup_test_db().await;
    let pool = db.pool();
    let clients = PgClientRepository::new(pool.clone());
    let cars = PgCarRepository::new(pool.clone());
    let repairs = PgRepairRepository::new(pool.clone());
    let parts = PgPartRepository::new(pool.clone());
    let repair_parts = PgRepairPartRepository::new(pool.clone());
    let stock_movements = PgStockMovementRepository::new(pool.clone());
    let client = fixtures::client(1);
    let car = fixtures::car(2, client.id());
    let mut repair = fixtures::repair(3, client.id(), car.id());
    let mut part = fixtures::part(4, "Oil filter", Some("of-123"), 10, 3);
    let original_quantity = part.quantity();
    let original_parts_price = repair.parts_price();
    let used_quantity = PartQuantity::new(2);

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");
    repairs.save(&repair).await.expect("repair should be saved");
    parts.save(&part).await.expect("part should be saved");

    let repair_part = RepairPart::new(
        RepairPartId::from_uuid(Uuid::from_u128(5)),
        repair.id(),
        part.id(),
        used_quantity,
        Money::byn_minor(1_000).expect("valid unit cost"),
        Money::byn_minor(2_500).expect("valid unit price"),
        fixtures::fixed_time(10),
    )
    .expect("valid repair part");
    let movement = StockMovement::new(
        StockMovementId::from_uuid(Uuid::from_u128(6)),
        part.id(),
        StockMovementType::Out,
        used_quantity,
        StockMovementReason::RepairUsage,
        None,
        fixtures::fixed_time(11),
        fixtures::fixed_time(12),
    )
    .expect("valid stock movement");
    part.decrease_stock(used_quantity, fixtures::fixed_time(20))
        .expect("part stock should decrease");
    repair
        .update_prices(
            Money::byn_minor(10_000).expect("valid labor price"),
            Money::byn_minor(5_000).expect("valid parts price"),
            Money::byn_minor(2_000).expect("valid parts cost"),
            fixtures::fixed_time(21),
        )
        .expect("repair prices should update");

    let uow = PgRepairPartUnitOfWork::begin(&pool)
        .await
        .expect("repair part unit of work should begin");
    uow.parts()
        .save(&part)
        .await
        .expect("part should be saved in transaction");
    uow.repair_parts()
        .save(&repair_part)
        .await
        .expect("repair part should be saved in transaction");
    uow.stock_movements()
        .save(&movement)
        .await
        .expect("stock movement should be saved in transaction");
    uow.repairs()
        .save(&repair)
        .await
        .expect("repair should be saved in transaction");
    uow.commit()
        .await
        .expect("repair part unit of work should commit");

    let loaded_part = parts
        .get(part.id())
        .await
        .expect("part should be loaded")
        .expect("part should exist");
    assert_eq!(
        loaded_part.quantity().value(),
        original_quantity.value() - used_quantity.value()
    );

    let loaded_repair_part = repair_parts
        .get(repair_part.id())
        .await
        .expect("repair part should be loaded")
        .expect("repair part should exist after commit");
    assert_eq!(loaded_repair_part, repair_part);

    let loaded_movement = stock_movements
        .get(movement.id())
        .await
        .expect("stock movement should be loaded")
        .expect("stock movement should exist after commit");
    assert_eq!(loaded_movement, movement);

    let loaded_repair = repairs
        .get(repair.id())
        .await
        .expect("repair should be loaded")
        .expect("repair should exist");
    assert_eq!(loaded_repair.parts_price(), repair.parts_price());
    assert_ne!(loaded_repair.parts_price(), original_parts_price);
}

#[tokio::test]
async fn repair_part_unit_of_work_rolls_back_all_changes() {
    let db = common::setup_test_db().await;
    let pool = db.pool();
    let clients = PgClientRepository::new(pool.clone());
    let cars = PgCarRepository::new(pool.clone());
    let repairs = PgRepairRepository::new(pool.clone());
    let parts = PgPartRepository::new(pool.clone());
    let repair_parts = PgRepairPartRepository::new(pool.clone());
    let stock_movements = PgStockMovementRepository::new(pool.clone());
    let client = fixtures::client(1);
    let car = fixtures::car(2, client.id());
    let mut repair = fixtures::repair(3, client.id(), car.id());
    let mut part = fixtures::part(4, "Oil filter", Some("of-123"), 10, 3);
    let original_quantity = part.quantity();
    let original_parts_price = repair.parts_price();
    let used_quantity = PartQuantity::new(2);

    clients.save(&client).await.expect("client should be saved");
    cars.save(&car).await.expect("car should be saved");
    repairs.save(&repair).await.expect("repair should be saved");
    parts.save(&part).await.expect("part should be saved");

    let repair_part = RepairPart::new(
        RepairPartId::from_uuid(Uuid::from_u128(5)),
        repair.id(),
        part.id(),
        used_quantity,
        Money::byn_minor(1_000).expect("valid unit cost"),
        Money::byn_minor(2_500).expect("valid unit price"),
        fixtures::fixed_time(10),
    )
    .expect("valid repair part");
    let movement = StockMovement::new(
        StockMovementId::from_uuid(Uuid::from_u128(6)),
        part.id(),
        StockMovementType::Out,
        used_quantity,
        StockMovementReason::RepairUsage,
        None,
        fixtures::fixed_time(11),
        fixtures::fixed_time(12),
    )
    .expect("valid stock movement");
    part.decrease_stock(used_quantity, fixtures::fixed_time(20))
        .expect("part stock should decrease");
    repair
        .update_prices(
            Money::byn_minor(10_000).expect("valid labor price"),
            Money::byn_minor(5_000).expect("valid parts price"),
            Money::byn_minor(2_000).expect("valid parts cost"),
            fixtures::fixed_time(21),
        )
        .expect("repair prices should update");

    let uow = PgRepairPartUnitOfWork::begin(&pool)
        .await
        .expect("repair part unit of work should begin");
    uow.parts()
        .save(&part)
        .await
        .expect("part should be saved in transaction");
    uow.repair_parts()
        .save(&repair_part)
        .await
        .expect("repair part should be saved in transaction");
    uow.stock_movements()
        .save(&movement)
        .await
        .expect("stock movement should be saved in transaction");
    uow.repairs()
        .save(&repair)
        .await
        .expect("repair should be saved in transaction");
    uow.rollback()
        .await
        .expect("repair part unit of work should roll back");

    let loaded_part = parts
        .get(part.id())
        .await
        .expect("part should be loaded")
        .expect("part should exist");
    assert_eq!(loaded_part.quantity(), original_quantity);

    let loaded_repair_part = repair_parts
        .get(repair_part.id())
        .await
        .expect("repair part lookup should succeed");
    assert!(loaded_repair_part.is_none());

    let loaded_movement = stock_movements
        .get(movement.id())
        .await
        .expect("stock movement lookup should succeed");
    assert!(loaded_movement.is_none());

    let loaded_repair = repairs
        .get(repair.id())
        .await
        .expect("repair should be loaded")
        .expect("repair should exist");
    assert_eq!(loaded_repair.parts_price(), original_parts_price);
}
