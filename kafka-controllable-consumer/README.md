# kafka-controllable-consumer

Test application to be deployed to the cluster in mirrord E2E tests.

A Kafka consumer whose consumption is driven over HTTP instead of running freely. The test deploys
it as the queue-splitting target, then decides when it reads and when it stalls.

Kafka splitting keeps the deployed workload reading from a temporary fallback topic while a session
is running, and drains that topic before tearing the split down. A consumer that reads as fast as it
can never leaves a backlog, so there is nothing to observe. Holding consumption back lets a test
build a known backlog, watch the operator wait for it, and assert the drain window behaves — that it
ends early once the topic is empty, and that it gives up when the configured cap elapses.

## API

| Method | Path       | Query                          | Description                                                                          |
| ------ | ---------- | ------------------------------ | ------------------------------------------------------------------------------------ |
| `GET`  | `/healthz` |                                | Readiness probe.                                                                     |
| `POST` | `/consume` | `count`, `wait_ms`             | Reads and commits up to `count` messages from the subscribed topic.                  |
| `POST` | `/peek`    | `topic`, `count`, `wait_ms`    | Reads up to `count` messages from `topic` **without committing**, leaving lag intact. |

`count` defaults to 1 and `wait_ms` to 5000. `/peek` exists so a test can prove messages are still
being forwarded without consuming them, which would otherwise shrink the very backlog under test.

Configured by `--address`/`--group`/`--topic`/`--port`, or the `ADDRESS`, `GROUP`, `TOPIC` and
`PORT` environment variables. The port defaults to 8080.
