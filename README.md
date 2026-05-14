# AutoServiceTelegramBot

## Running garage-telegram locally

1. Create local env file:
   ```bash
   cp .env.example .env
   ```
2. Set `TELEGRAM_BOT_TOKEN` in `.env`.
3. Set `DATABASE_URL` in `.env`.
4. Start PostgreSQL:
   ```bash
   docker compose up -d
   ```
5. Run migrations if they are not applied automatically:
   ```bash
   DATABASE_URL=postgres://garage:garage@localhost:5432/garage sqlx migrate run --source crates/garage-infra/migrations
   ```
6. Run the bot:
   ```bash
   cargo run -p garage-telegram
   ```

For a private launch, set `OWNER_CHAT_ID` to your Telegram chat/user id. If it is empty, the bot accepts messages from anyone and logs a startup warning.

## Docker

See [docs/DOCKER.md](docs/DOCKER.md) for the Docker Compose setup with PostgreSQL, migrations, and the Telegram bot.

Important: use `garage` for the bot and `garage_test` for tests. Do not run `docker compose down -v` unless you intentionally want to delete the PostgreSQL volume.
