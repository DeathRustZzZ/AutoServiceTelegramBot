use sqlx::PgPool;
use std::path::Path;

/// Подготавливает реальную PostgreSQL базу для integration tests.
///
/// Требования перед запуском:
/// - поднять локальную БД, например `docker compose up -d db`;
/// - задать `DATABASE_URL`, например
///   `postgres://garage:garage@localhost:5432/garage`.
pub async fn setup_test_db() -> PgPool {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");

    let pool = garage_infra::db::pool::create_pool(&database_url)
        .await
        .expect("failed to create PostgreSQL pool");

    let migrations_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    let migrator = sqlx::migrate::Migrator::new(migrations_path)
        .await
        .expect("failed to load garage-infra migrations");

    migrator
        .run(&pool)
        .await
        .expect("failed to run garage-infra migrations");

    sqlx::query(
        r#"
        TRUNCATE TABLE
            stock_movements,
            payments,
            repair_parts,
            repairs,
            part_supplies,
            bookings,
            cars,
            parts,
            clients
        RESTART IDENTITY CASCADE
        "#,
    )
    .execute(&pool)
    .await
    .expect("failed to truncate test database tables");

    pool
}
