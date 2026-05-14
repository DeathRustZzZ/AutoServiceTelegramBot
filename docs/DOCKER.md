# Docker Setup

## Services

Docker Compose starts three containers:

- `postgres`: PostgreSQL 16 with persistent data in `garage_postgres_data`.
- `garage-migrate`: one-shot `sqlx migrate run` container.
- `garage-telegram`: Telegram bot runtime container.

The production database is `garage`. The test database is `garage_test`.

## Run Locally

1. Create `.env`:
   ```bash
   cp .env.example .env
   ```
2. Fill `TELEGRAM_BOT_TOKEN` and `OWNER_CHAT_ID` in `.env`.
3. Build and start:
   ```bash
   docker compose up -d --build
   ```
4. Watch bot logs:
   ```bash
   docker compose logs -f garage-telegram
   ```
5. Stop services:
   ```bash
   docker compose down
   ```

Do not run `docker compose down -v` unless you intentionally want to delete PostgreSQL data. The `-v` flag removes the `garage_postgres_data` volume, including clients, repairs, stock, payments, and bookings.

## Database

Open `psql`:

```bash
docker compose exec postgres psql -U garage -d garage
```

Back up the production database:

```bash
docker compose exec postgres pg_dump -U garage garage > backup.sql
```

Restore a backup:

```bash
cat backup.sql | docker compose exec -T postgres psql -U garage -d garage
```

## Tests

Do not run tests against the production `garage` database.

Use the separate `garage_test` database:

```bash
DATABASE_URL=postgres://garage:garage@localhost:5432/garage_test cargo test -p garage-infra
```

The compose setup mounts `docker/postgres/init`, which creates `garage_test` on first volume initialization. PostgreSQL init scripts run only when the volume is created. If you already have an existing volume, create the test database manually:

```bash
docker compose exec postgres createdb -U garage garage_test
```

If PostgreSQL reports a template collation warning on an older existing volume, create the test database from `template0`:

```bash
docker compose exec postgres createdb -U garage -T template0 garage_test
```

## Migrations

Migrations live in `crates/garage-infra/migrations`.

Compose runs them automatically through `garage-migrate` before starting `garage-telegram`.

Manual run:

```bash
docker compose run --rm garage-migrate migrate run --source crates/garage-infra/migrations
```
