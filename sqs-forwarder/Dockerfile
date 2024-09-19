# Using the agent's build image, that has rustup targets, zigbuild etc.
FROM --platform=$BUILDPLATFORM ghcr.io/metalbear-co/ci-agent-build:latest AS build-env
ARG TARGETARCH

WORKDIR /build
COPY . .

# Translate docker's platform to cargo's target.
RUN /build/platform.sh


# Build binary.
ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo zigbuild -Z bindeps --target $(cat /.platform) --release
RUN cp /build/target/$(cat /.platform)/release/sqs-forwarder /sqs-forwarder

# Slim final layer with only the binary.
FROM debian:stable-slim
# TODO: optimize this?
RUN apt-get update
RUN apt-get install -y --no-install-recommends ca-certificates
RUN update-ca-certificates

WORKDIR /app
COPY --from=build-env /sqs-forwarder .

CMD ["./sqs-forwarder"]

