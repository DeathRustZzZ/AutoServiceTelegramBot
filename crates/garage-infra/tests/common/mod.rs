use sqlx::{PgConnection, PgPool};
use std::path::Path;

/// Подготавливает реальную PostgreSQL базу для integration tests.
///
/// Требования перед запуском:
/// - поднять локальную БД, например `docker compose up -d db`;
/// - задать `DATABASE_URL`, например
///   `postgres://garage:garage@localhost:5432/garage`.
pub struct TestDb {
    pool: PgPool,
    _lock_connection: PgConnection,
}

impl TestDb {
    pub fn pool(&self) -> PgPool {
        self.pool.clone()
    }
}

pub async fn setup_test_db() -> TestDb {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");

    let pool = garage_infra::db::pool::create_pool(&database_url)
        .await
        .expect("failed to create PostgreSQL pool");
    let mut lock_connection = sqlx::Connection::connect(&database_url)
        .await
        .expect("failed to create PostgreSQL advisory lock connection");

    sqlx::query("SELECT pg_advisory_lock(42)")
        .execute(&mut lock_connection)
        .await
        .expect("failed to acquire PostgreSQL advisory lock for integration test");

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

    TestDb {
        pool,
        _lock_connection: lock_connection,
    }
}
