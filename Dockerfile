FROM lukemathwalker/cargo-chef:latest-rust-latest AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.lock .
COPY Cargo.toml .
COPY /src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
ENV SQLX_OFFLINE='true'
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY Cargo.lock .
COPY Cargo.toml .
COPY /.sqlx ./.sqlx
COPY /.env ./.env
COPY /migrations ./migrations
COPY /src ./src
RUN mkdir -p data
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=builder --chown=nonroot:nonroot /app/target/release/pka_site_backend .
COPY --from=builder --chown=nonroot:nonroot /app/data ./data
COPY --from=builder --chown=nonroot:nonroot /app/migrations ./migrations
ENV DATABASE_URL=sqlite://./pka_index_data/pka_db.sqlite3
USER nonroot
ENTRYPOINT ["/app/pka_site_backend"]

