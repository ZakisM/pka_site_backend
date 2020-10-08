FROM rust:latest as builder

RUN USER=root cargo new --bin pka_site_backend

ENV RUSTFLAGS='-C target-cpu=znver2'

WORKDIR ./pka_site_backend

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN apt-get update \
    && apt-get install -y cmake \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release
RUN rm src/*.rs

ADD ./src ./src

RUN rm ./target/release/deps/pka_site_backend*
RUN cargo build --release

ADD ./pka_db.sqlite3 ./pka_db.sqlite3

FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /pka_site_backend/target/release/pka_site_backend ${APP}/pka_site_backend
COPY --from=builder /pka_site_backend/pka_db.sqlite3 ${APP}/pka_db.sqlite3

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./pka_site_backend"]