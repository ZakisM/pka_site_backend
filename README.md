![Rust](https://github.com/ZakisM/pka_site_backend/workflows/Rust/badge.svg)

# pka_site_backend
Download rust: https://rustup.rs/
Install Diesel CLI for making changes to DB: https://github.com/diesel-rs/diesel/tree/master/diesel_cli

To run in release (optimized) mode: run 'cargo run --release' from project root.
To run in debug mode: run 'cargo run' from project root.
To check for errors (quicker than running): run 'cargo check' from project root.

Building with docker:

docker build -t zakism/painkiller-already-index:latest .

Running with docker:

docker-compose up -d
