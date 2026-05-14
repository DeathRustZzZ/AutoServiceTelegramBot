mod common;

use chrono::TimeZone;
use garage_app::ClientRepository;
use garage_domain::{Client, ClientId, ClientName, ClientNotes, PhoneNumber};
use garage_infra::repositories::client::PgClientRepository;
use uuid::Uuid;

#[tokio::test]
async fn client_repository_saves_and_loads_client() {
    let pool = common::setup_test_db().await;
    let repository = PgClientRepository::new(pool);
    let now = chrono::Utc
        .with_ymd_and_hms(2026, 5, 14, 9, 0, 0)
        .single()
        .expect("valid fixed timestamp");

    let client = Client::new(
        ClientId::from_uuid(Uuid::from_u128(1)),
        ClientName::parse("Иван Петров").expect("valid client name"),
        PhoneNumber::parse("+375291234567").expect("valid phone number"),
        ClientNotes::parse("Постоянный клиент").expect("valid client notes"),
        now,
    );

    repository
        .save(&client)
        .await
        .expect("client should be saved");

    let loaded = repository
        .get(client.id())
        .await
        .expect("client should be loaded")
        .expect("client should exist");

    assert_eq!(loaded, client);
}
