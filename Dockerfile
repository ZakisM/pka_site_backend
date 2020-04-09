FROM ekidd/rust-musl-builder:latest as build

WORKDIR /pka_site_backend

USER root

RUN chown -R rust:rust /pka_site_backend

USER rust

# copy over manifests
ADD --chown=rust:rust ./Cargo.lock ./Cargo.lock
ADD --chown=rust:rust ./Cargo.toml ./Cargo.toml

ADD --chown=rust:rust ./src ./src

RUN cargo build --release

ADD --chown=rust:rust ./front-end ./front-end

ADD --chown=rust:rust pka_db.sqlite3 pka_db.sqlite3

FROM alpine:latest

RUN apk --no-cache add ca-certificates

WORKDIR /pka_site_backend

# copy build artifact from previous stage
COPY --from=build /pka_site_backend/target/x86_64-unknown-linux-musl/release/pka_site_backend .
COPY --from=build /pka_site_backend/front-end ./front-end
COPY --from=build /pka_site_backend/pka_db.sqlite3 .

CMD ["./pka_site_backend"]
