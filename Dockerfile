# fetch the vendor with the builder platform to avoid qemu issues
FROM --platform=$BUILDPLATFORM rust:1 AS vendors

WORKDIR /code

RUN cargo init

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

RUN mkdir -p ./.cargo \
  && cargo vendor > ./.cargo/config

FROM rust:1 as builder

COPY --from=vendors /code/.cargo /code/.cargo
COPY --from=vendors /code/vendor /code/vendor

WORKDIR /code

COPY . .

RUN cargo install --path . --offline

FROM debian:buster-slim

#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/oura /usr/local/bin/oura

CMD ["oura"]
