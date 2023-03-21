# Using the agent's build image, that has rustup targets, zigbuild etc.
FROM --platform=$BUILDPLATFORM ghcr.io/metalbear-co/ci-agent-build:latest as build-env
ARG TARGETARCH

WORKDIR /build
COPY . .

# Translate docker's platform to cargo's target.
RUN /build/platform.sh


# Build binary.
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo zigbuild -Z bindeps --target $(cat /.platform) --release
RUN cp /build/target/$(cat /.platform)/release/unix-socket-server /unix-socket-server

# Slim final layer with only the binary.
FROM debian:stable-slim
WORKDIR /app
COPY --from=build-env /unix-socket-server .

CMD ["./unix-socket-server"]

