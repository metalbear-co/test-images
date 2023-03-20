FROM --platform=$BUILDPLATFORM ghcr.io/metalbear-co/ci-agent-build:latest as build-env
ARG TARGETARCH

WORKDIR /build
COPY . .

RUN /build/platform.sh


ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo zigbuild -Z bindeps --target $(cat /.platform) --release
RUN cp /build/target/$(cat /.platform)/release/unix-socket-server /unix-socket-server

FROM debian:stable-slim
WORKDIR /app
COPY --from=build-env /unix-socket-server .

CMD ["./unix-socket-server"]

