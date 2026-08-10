# Using the agent's build image, that has rustup targets, zigbuild etc.
FROM --platform=$BUILDPLATFORM ghcr.io/metalbear-co/ci-agent-build:latest AS build-env
ARG TARGETARCH

# librdkafka is built from source by the `cmake-build` feature, which needs a C/C++ toolchain and
# SASL headers on top of what the build image carries.
RUN apt-get update \
    && apt-get install -y --no-install-recommends cmake clang libsasl2-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY . .

# Translate docker's platform to cargo's target.
RUN /build/platform.sh

# Build binary.
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo zigbuild --target $(cat /.platform) --release
RUN cp /build/target/$(cat /.platform)/release/kafka-controllable-consumer /kafka-controllable-consumer

# Slim final layer with only the binary.
FROM debian:stable-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libsasl2-2 \
    && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates

WORKDIR /app
COPY --from=build-env /kafka-controllable-consumer .

CMD ["./kafka-controllable-consumer"]
