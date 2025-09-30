# Repository Guidelines

## Project Structure & Module Organization
Core service code lives under `src/`, with `main.rs` wiring an Axum `Router` from `routes/`, request handlers in `handlers/`, and shared `sqlx` helpers in `db.rs` plus domain models in `models/`. Background ingestion and sync jobs sit in `updater/` and `workers/`; Redis helpers are in `redis_db.rs`, and shared utilities under `conduit/` and `search/`. Database schema migrations (SQLite) live in `migrations/`. Assets for local infra (nginx TLS, docker) are at the repository root: `nginx.conf`, `docker-compose.yml`, `dump.rdb`, and the precomputed `pka_index_data/`.

## Build, Test, and Development Commands
- `cargo run` — Start the API with debug logging.
- `cargo run --release` — Optimized build; mirrors Docker image behavior.
- `cargo check` — Fast type and borrow validation before committing.
- `cargo test` — Run unit tests embedded in modules.
- `cargo sqlx prepare -- --bin pka_site_backend` — Regenerate `.sqlx/` metadata after changing queries or migrations (requires `DATABASE_URL`).
- `docker-compose up -d` — Bring up backend plus Redis via containers for end-to-end smoke tests.

## Coding Style & Naming Conventions
Rust code should stay `cargo fmt --all` clean (rustfmt's default 4-space indentation, trailing commas, and module ordering). Favor `snake_case` for modules, files, and functions; use `CamelCase` for structs/enums that back API payloads or SQLx row mappings. Keep Axum handlers small, prefer returning `Result<impl IntoResponse, ApiError>`, and bubble errors via `ApiError`. External API keys and secrets belong in `.env` (loaded through `dotenv`) rather than hard-coded constants.

## Testing Guidelines
Unit tests live next to their sources inside `#[cfg(test)]` modules (e.g., `src/models/sitemap_xml.rs`). Name tests descriptively with `test_*`. Use deterministic fixtures—clone or stub YouTube responses rather than hitting the network. Run `cargo test` before every PR; add targeted tests for new routes, schema changes, or data transforms. When altering SQL migrations, add assertions that cover the new fields.

## Commit & Pull Request Guidelines
Commits follow short, imperative summaries (`Add libsqlite`, `Update dependencies`). Keep related changes squashed together; include migration hashes or new env vars in the body. Pull requests should outline intent, list any manual test commands, and call out schema or cache impacts. Link tracking issues when available and attach screenshots only when the API surface affects rendered responses.

## Configuration & Runtime Notes
Local development expects a Redis instance on `localhost:6379` and `YT_API_KEY` in the environment; `dotenv` will populate it for `cargo run`. For HTTPS testing, reuse the self-signed certificate flow described in `README.md`, updating `nginx.conf` paths to match your host.
