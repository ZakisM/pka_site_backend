# Repository Guidelines

## Project Structure & Module Organization
Core service code lives under `src/`, with `main.rs` starting the server and `routes/` assembling the whole HTTP app (CORS, compression, `Cache-Control`). Every request is answered from memory: `catalog/` holds a snapshot of every episode, video and moment, plus the search (`catalog/search.rs`), related-moments index (`catalog/related.rs`) and prerendered sitemap and episode list. Handlers in `handlers/` read only from that snapshot and never touch SQLite. The one background task in `workers/` checks YouTube every five minutes through `updater/`, saves each new episode in one transaction, and rebuilds the catalog when something was added (and hourly regardless). SQLite queries live under `conduit/`; schema migrations in `migrations/`, which the server applies itself at startup, so a schema change ships as a new migration file and never as SQL run on the server by hand. Never edit a migration once it has shipped: sqlx checks applied migrations haven't changed, and the server would refuse to start; add a new one instead. Once a day `updater/sync.rs` checks every stored episode against YouTube, as its API terms require (nothing kept unchecked past 30 days): titles and lengths follow YouTube; moments follow the video description when it has a timeline, and are kept when it has none or the episode is locked (`moments_locked`, or a hand-added part with a fractional number); an episode whose video has gone is hidden, then deleted after a week. Assets for local infra (nginx TLS, docker) are at the repository root: `nginx.conf`, `docker-compose.yml`, and the precomputed `pka_index_data/`.

## Build, Test, and Development Commands
- `cargo run` — Start the API with debug logging.
- `cargo run --release` — Optimized build; mirrors Docker image behavior.
- `cargo check` — Fast type and borrow validation before committing.
- `cargo test` — Run the unit tests, plus `src/http_tests.rs`: every endpoint end to end against an in-memory database built from `migrations/`.
- `cargo sqlx prepare -- --all-targets` — Regenerate `.sqlx/` metadata after changing queries or migrations (requires `DATABASE_URL`).
- `docker-compose up -d` — Bring up the backend and frontend containers for end-to-end smoke tests.
- `EXPOSE_OPENAPI=true cargo run` — also serves `GET /openapi.json` for generated OpenAPI docs.

## Coding Style & Naming Conventions
Rust code should stay `cargo fmt --all` clean (rustfmt's default 4-space indentation, trailing commas, and module ordering). Favor `snake_case` for modules, files, and functions; use `CamelCase` for structs/enums that back API payloads or SQLx row mappings. Keep Axum handlers small, prefer returning `Result<impl IntoResponse, ApiError>`, and bubble errors via `ApiError`. External API keys and secrets belong in `.env` (loaded through `dotenv`) rather than hard-coded constants.

## Testing Guidelines
Unit tests live next to their sources inside `#[cfg(test)]` modules (e.g., `src/catalog/search.rs`); HTTP behaviour (status codes, JSON shapes, caching headers) is tested in `src/http_tests.rs`. Name tests descriptively with `test_*`. Use deterministic fixtures—clone or stub YouTube responses rather than hitting the network. Run `cargo test` before every PR; add targeted tests for new routes, schema changes, or data transforms. When altering SQL migrations, add assertions that cover the new fields.

## Commit & Pull Request Guidelines
Commits follow short, imperative summaries (`Add libsqlite`, `Update dependencies`). Keep related changes squashed together; include migration hashes or new env vars in the body. Pull requests should outline intent, list any manual test commands, and call out schema or cache impacts. Link tracking issues when available and attach screenshots only when the API surface affects rendered responses.

## Configuration & Runtime Notes
Local development expects `YT_API_KEY` in the environment; `dotenv` will populate it for `cargo run`. For HTTPS testing, reuse the self-signed certificate flow described in `README.md`, updating `nginx.conf` paths to match your host.
