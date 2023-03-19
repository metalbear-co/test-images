FROM rust:1.68-slim as base

WORKDIR /build
COPY . .

RUN cargo build

FROM debian:stable-slim
WORKDIR /app
COPY --from=base /build/target/debug/unix-socket-server .

CMD ["./unix-socket-server"]
