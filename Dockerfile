# select build image
FROM rust as build

# create a new empty shell project
RUN USER=root cargo new --bin pka_site_backend
WORKDIR /pka_site_backend

# copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/pka_site_backend*
RUN cargo build --release

# copy front-end
COPY ./front-end ./front-end

# copy database items
COPY pka_db.sqlite3 pka_db.sqlite3

# for optimizing image size
FROM rust

WORKDIR /pka_site_backend

# copy build artifact from previous stage
COPY --from=build /pka_site_backend/target/release/pka_site_backend .
COPY --from=build /pka_site_backend/front-end ./front-end
COPY --from=build /pka_site_backend/pka_db.sqlite3 .

CMD ["./pka_site_backend"]