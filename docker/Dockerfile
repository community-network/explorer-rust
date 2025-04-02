FROM rust:1.83 AS builder
WORKDIR /usr/src/myapp
COPY . .
ARG github_token
ARG CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN apt-get update -y && apt-get upgrade -y
RUN apt-get install protobuf-compiler libprotobuf-dev -y && apt-get clean
RUN git config --global credential.helper store && echo "https://zefanjajobse:${github_token}@github.com" > ~/.git-credentials && cargo install --path .

FROM debian:bookworm-slim

# Install Opn SSL libs

RUN apt-get update -y && apt-get install pkg-config libssl-dev -y

HEALTHCHECK --interval=5m --timeout=3s --start-period=5s \
  CMD curl -f http://127.0.0.1:3030/ || exit 1

COPY server.pem .
COPY --from=builder /usr/local/cargo/bin/standalone /usr/local/bin/standalone
COPY --from=builder /usr/local/cargo/bin/rabbit_host /usr/local/bin/rabbit_host
COPY --from=builder /usr/local/cargo/bin/rabbit_worker /usr/local/bin/rabbit_worker
RUN apt-get update && apt-get upgrade -y && apt-get install --assume-yes curl protobuf-compiler libprotobuf-dev libmariadb-dev libpq-dev && apt-get clean
CMD ["standalone"]
