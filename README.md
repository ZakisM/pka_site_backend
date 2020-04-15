![Rust](https://github.com/ZakisM/pka_site_backend/workflows/Rust/badge.svg)

# PKA Index Backend

This repo holds the backend code for https://www.pkaindex.com.

To run locally:

#### To develop/test
1. Download rust: https://rustup.rs/.
2. Install Diesel CLI for making changes to DB: https://github.com/diesel-rs/diesel/tree/master/diesel_cli

- To run in release (optimized) mode: run `cargo run --release` from project root. 
- To run in debug mode: run `cargo run` from project root. 
- To check for errors (quicker than running): run `cargo check` from project root.


#### Test With Docker - Note this is creating an optimized build so not suitable for development.
1. `docker build -t zakism/pka-index-backend:latest .`
2. `docker run -p 1234:1234 zakism/pka-index-backend:latest`
3. Backend should now be serving an API from http://0.0.0.0:1234.

### Alternative way to test backend and frontend with docker:

1. `docker-compose up -d`

