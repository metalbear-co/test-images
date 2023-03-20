FROM --platform=$BUILDPLATFORM debian:stable-slim as base
ARG TARGETARCH

# Run the platform script to translate docker target name
# to cargo target name. Then copy the correct binary to
# /binary/unix-socket-server.
WORKDIR /binary
COPY platform.sh .
RUN ./platform.sh

COPY target target
RUN pwd
RUN ls
RUN ls target
RUN ls target/*/release
RUN cp target/$(cat /.platform)/release/unix-socket-server .

# Now copy only the binary into a new layer.
FROM --platform=$BUILDPLATFORM debian:stable-slim
WORKDIR /app
COPY --from=base /binary/unix-socket-server .
RUN ls


CMD ["./unix-socket-server"]

