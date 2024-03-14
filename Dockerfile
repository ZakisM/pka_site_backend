FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.lock .
COPY Cargo.toml .
COPY /src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.lock .
COPY Cargo.toml .
COPY /.env ./.env
COPY /migrations ./migrations
COPY /src ./src
RUN mkdir -p data
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
WORKDIR /app
COPY --from=builder /app/target/release/pka_site_backend .
COPY --from=builder /app/.env .
COPY --from=builder /app/data ./data
COPY --from=builder /app/migrations ./migrations
ENTRYPOINT ["/app/pka_site_backend"]

