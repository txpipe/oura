FROM rust:1 as builder

WORKDIR /code

COPY . .

RUN cargo install --path .

FROM debian:buster-slim

#RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/oura /usr/local/bin/oura

CMD ["oura"]
