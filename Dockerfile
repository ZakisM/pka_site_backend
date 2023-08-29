ARG BASE_IMAGE=rust:latest

FROM $BASE_IMAGE as builder
WORKDIR /app
RUN mkdir data
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./.env ./.env
COPY ./pka_db.sqlite3 ./pka_db.sqlite3
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/pka_site_backend /
COPY --from=builder /app/.env /
COPY --from=builder /app/data data
COPY --from=builder /app/pka_db.sqlite3 /data/pka_db.sqlite3
CMD ["./pka_site_backend"]
