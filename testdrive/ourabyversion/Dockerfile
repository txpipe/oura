FROM --platform=linux/amd64 rust:1-buster as builder

ARG RELEASE_TAG

RUN git config --global init.defaultBranch main &&  \
    git config --global advice.detachedHead false && \
    git init && \
    git remote add origin https://github.com/txpipe/oura.git && \
    git fetch  && \
    test -z ${RELEASE_TAG} &&  GITREV=$(git describe --tags $(git rev-list --tags --max-count=1)) || GITREV=${RELEASE_TAG}  && \
    git checkout "${GITREV}"

RUN cargo build --release --target x86_64-unknown-linux-gnu --all-features && \
    cp /target/x86_64-unknown-linux-gnu/release/oura /oura

# Run app
FROM debian:buster-slim

COPY --from=builder /oura /usr/local/bin/oura

ENTRYPOINT [ "oura" ]
