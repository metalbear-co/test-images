Simple HTTP/HTTPS server written in Go.

Always responds with 200 OK.

# Configuration:
* `SERVER_MESSAGE` - message to respond with. Optional, defaults to `Hello from remote!`.
* `SERVER_MODE` - either `HTTP` or `HTTPS`. Optional, defaults to `HTTP`.
* `SERVER_PORT` - port to listen on. Optional, defaults to `80` in the `HTTP` mode and `443` in the `HTTPS` mode.
* `TLS_SERVER_CERT` - path to a PEM file containing certificate chain to use for server authentication (required in the `HTTPS` mode).
* `TLS_SERVER_KEY` - path to a PEM file containing private key to use with the certificate chain from `TLS_SERVER_CERT` (required in the `HTTPS` mode).
* `TLS_CLIENT_ROOTS` - path to a PEM file containing certificates to use as trusted roots for client authentication. Optional, if mode is `HTTPS` and this variable is not provided, the server will not offer client authentication.
