FROM --platform=$BUILDPLATFORM rust:1.68-slim as base
ARG TARGETARCH

# Install docker inside this container, for cross.
RUN apt-get update
RUN apt-get install -y curl
RUN curl -fsSL https://get.docker.com -o get-docker.sh
RUN sh get-docker.sh
RUN dockerd


# Install cross.
ENV CROSS_CONTAINER_IN_CONTAINER=true
RUN cargo install cross --git https://github.com/cross-rs/cross

WORKDIR /build
COPY . .

# Stores the platform string in /.platform.
RUN ./platform.sh

RUN rustup target add $(cat /.platform)

# Build the binary.
ENV CARGO_TARGET_DIR=/build
RUN cross build --target $(cat /.platform) --release


# Now create a slim final layer with only the binary.
FROM debian:stable-slim
WORKDIR /app
COPY --from=base /build/unix-socket-server .

CMD ["./unix-socket-server"]

