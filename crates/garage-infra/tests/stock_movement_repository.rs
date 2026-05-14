mod common;

use common::fixtures;
use garage_app::{PartRepository, StockMovementRepository};
use garage_domain::{
    PartQuantity, StockMovement, StockMovementId, StockMovementReason, StockMovementType,
};
use garage_infra::repositories::part::PgPartRepository;
use garage_infra::repositories::stock_movement::PgStockMovementRepository;
use uuid::Uuid;

#[tokio::test]
async fn stock_movement_repository_saves_loads_and_lists_by_part() {
    let db = common::setup_test_db().await;
    let parts = PgPartRepository::new(db.pool());
    let stock_movements = PgStockMovementRepository::new(db.pool());
    let part = fixtures::part(1, "Oil filter", Some("of-123"), 10, 3);

    parts.save(&part).await.expect("part should be saved");

    let movement = StockMovement::new(
        StockMovementId::from_uuid(Uuid::from_u128(2)),
        part.id(),
        StockMovementType::Out,
        PartQuantity::new(2),
        StockMovementReason::RepairUsage,
        None,
        fixtures::fixed_time(10),
        fixtures::fixed_time(20),
    )
    .expect("valid stock movement");

    stock_movements
        .save(&movement)
        .await
        .expect("stock movement should be saved");

    let loaded = stock_movements
        .get(movement.id())
        .await
        .expect("stock movement should be loaded")
        .expect("stock movement should exist");
    assert_eq!(loaded, movement);

    let listed = stock_movements
        .list_by_part(part.id())
        .await
        .expect("stock movements should be listed by part");
    assert!(listed.contains(&movement));

    let part_after_movement = parts
        .get(part.id())
        .await
        .expect("part should be loaded after stock movement")
        .expect("part should still exist");
    assert_eq!(part_after_movement.quantity(), part.quantity());
}
