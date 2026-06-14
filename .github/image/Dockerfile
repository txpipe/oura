FROM debian:12-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

ARG TARGETARCH
COPY bin/oura-Linux-${TARGETARCH} /bin/oura
RUN chmod +x /bin/oura
RUN ln -s /bin/oura /oura

ENTRYPOINT [ "oura" ]
