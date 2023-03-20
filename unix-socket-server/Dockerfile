FROM --platform=$BUILDPLATFORM docker:dind as base
ARG TARGETARCH

# Install Rust.
RUN apk update && apk add --no-cache curl gcc musl-dev
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

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

