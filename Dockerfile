ARG BASE_IMAGE=rust:latest
ARG BASE_CHEF_IMAGE=lukemathwalker/cargo-chef:latest

FROM $BASE_CHEF_IMAGE as planner
WORKDIR app
ENV RUSTFLAGS='-C target-cpu=znver2'
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM $BASE_CHEF_IMAGE as cacher
WORKDIR app
ENV RUSTFLAGS='-C target-cpu=znver2'
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM $BASE_IMAGE as builder
WORKDIR app
ENV RUSTFLAGS='-C target-cpu=znver2'
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./pka_db.sqlite3 ./pka_db.sqlite3
# Copy over the cached dependencies
COPY --from=cacher /app/target target
COPY --from=cacher $CARGO_HOME $CARGO_HOME
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11
COPY --from=builder /app/target/release/pka_site_backend /
COPY --from=builder /app/pka_db.sqlite3 /pka_db.sqlite3
CMD ["./pka_site_backend"]
