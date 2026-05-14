mod common;

use common::fixtures;
use garage_app::{PartRepository, PartSupplyRepository};
use garage_domain::{PartQuantity, PartSupplier, PartSupply, PartSupplyId};
use garage_infra::repositories::part::PgPartRepository;
use garage_infra::repositories::part_supply::PgPartSupplyRepository;
use uuid::Uuid;

#[tokio::test]
async fn part_supply_repository_saves_loads_and_lists_by_part() {
    let db = common::setup_test_db().await;
    let parts = PgPartRepository::new(db.pool());
    let supplies = PgPartSupplyRepository::new(db.pool());
    let part = fixtures::part(1, "Oil filter", Some("of-123"), 10, 3);

    parts.save(&part).await.expect("part should be saved");

    let supply = PartSupply::new(
        PartSupplyId::from_uuid(Uuid::from_u128(2)),
        part.id(),
        PartQuantity::new(5),
        fixtures::fixed_time(3_600),
        PartSupplier::parse("Минск Склад").expect("valid supplier"),
        None,
        fixtures::fixed_time(10),
    )
    .expect("valid part supply");

    supplies
        .save(&supply)
        .await
        .expect("part supply should be saved");

    let loaded = supplies
        .get(supply.id())
        .await
        .expect("part supply should be loaded")
        .expect("part supply should exist");
    assert_eq!(loaded, supply);

    let listed = supplies
        .list_by_part(part.id())
        .await
        .expect("part supplies should be listed by part");
    assert!(listed.contains(&supply));

    let part_after_supply = parts
        .get(part.id())
        .await
        .expect("part should be loaded after supply")
        .expect("part should still exist");
    assert_eq!(part_after_supply.quantity(), part.quantity());
}
