FROM rust:1.68-slim as base

RUN cargo install cargo-zigbuild

WORKDIR /build
COPY . .
RUN ./mirrord/agent/platform.sh
RUN rustup target add $(/.platform)


ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
ENV CARGO_TARGET_DIR=/build
RUN cargo zigbuild -Z bindeps --target $(/.platform) --release

FROM debian:stable-slim
WORKDIR /app
COPY --from=base /build/unix-socket-server .

CMD ["./unix-socket-server"]

