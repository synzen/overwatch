FROM rust:1.79 as build

# create a new empty shell project
RUN USER=root cargo new --bin overwatch-api
WORKDIR /overwatch-api

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm -rf ./target/release*
RUN cargo build --release

# our final base
FROM rust:1.79-slim

# copy the build artifact from the build stage
COPY --from=build /overwatch-api/target/release/overwatch-api .

# set the startup command to run your binary
CMD ["./overwatch-api"]
