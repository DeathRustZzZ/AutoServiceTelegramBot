mod common;

use chrono::{DateTime, TimeZone, Utc};
use garage_app::PartRepository;
use garage_domain::{Money, Part, PartId, PartName, PartQuantity, PartSku};
use garage_infra::repositories::part::PgPartRepository;
use uuid::Uuid;

fn fixed_time(seconds: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 5, 14, 10, 0, seconds)
        .single()
        .expect("valid fixed timestamp")
}

fn part(id: u128, name: &str, sku: Option<&str>, quantity: u32, min_quantity: u32) -> Part {
    Part::new(
        PartId::from_uuid(Uuid::from_u128(id)),
        PartName::parse(name).expect("valid part name"),
        sku.map(PartSku::parse)
            .transpose()
            .expect("valid part sku")
            .flatten(),
        PartQuantity::new(quantity),
        PartQuantity::new(min_quantity),
        Money::byn_minor(2_500).expect("valid money"),
        None,
        fixed_time(id as u32),
    )
}

#[tokio::test]
async fn part_repository_saves_and_loads_part() {
    let db = common::setup_test_db().await;
    let repository = PgPartRepository::new(db.pool());
    let part = part(1, "Oil filter", Some("of-123"), 10, 3);

    repository.save(&part).await.expect("part should be saved");

    let loaded = repository
        .get(part.id())
        .await
        .expect("part should be loaded")
        .expect("part should exist");

    assert_eq!(loaded, part);
}

#[tokio::test]
async fn part_repository_search_finds_by_name_or_sku() {
    let db = common::setup_test_db().await;
    let repository = PgPartRepository::new(db.pool());
    let oil_filter = part(1, "Oil filter", Some("of-123"), 10, 3);
    let brake_pads = part(2, "Brake pads", Some("bp-456"), 6, 2);

    repository
        .save(&oil_filter)
        .await
        .expect("oil filter should be saved");
    repository
        .save(&brake_pads)
        .await
        .expect("brake pads should be saved");

    let by_name = repository
        .search("oil")
        .await
        .expect("parts should be searchable by name");
    assert!(by_name.contains(&oil_filter));
    assert!(!by_name.contains(&brake_pads));

    let by_sku = repository
        .search("BP-456")
        .await
        .expect("parts should be searchable by sku");
    assert!(by_sku.contains(&brake_pads));
    assert!(!by_sku.contains(&oil_filter));
}

#[tokio::test]
async fn part_repository_list_low_stock_returns_only_active_low_stock_parts() {
    let db = common::setup_test_db().await;
    let repository = PgPartRepository::new(db.pool());
    let active_low_stock = part(1, "Oil filter", Some("of-123"), 2, 3);
    let active_not_low_stock = part(2, "Brake pads", Some("bp-456"), 6, 2);
    let mut archived_low_stock = part(3, "Air filter", Some("af-789"), 1, 5);
    archived_low_stock
        .archive(fixed_time(10))
        .expect("part should be archived");

    repository
        .save(&active_low_stock)
        .await
        .expect("active low-stock part should be saved");
    repository
        .save(&active_not_low_stock)
        .await
        .expect("active non-low-stock part should be saved");
    repository
        .save(&archived_low_stock)
        .await
        .expect("archived low-stock part should be saved");

    let low_stock = repository
        .list_low_stock()
        .await
        .expect("low-stock parts should be listed");

    assert_eq!(low_stock, vec![active_low_stock]);
}
