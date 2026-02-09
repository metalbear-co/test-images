Simple app that consumes records from multiple Kafka topics,
using both the standard API and the Kafka Streams API.

The set of input topics is defined at runtime with environment variables.

# Configuration

At startup, the app finds all environment variables with keys following the pattern `INPUT_TOPIC_NAME_{id}`,
for example `INPUT_TOPIC_NAME_1` or `INPUT_TOPIC_NAME_some-name`.
The value of each such variable is assumed to be a Kafka topic name,
and the application consumes records from all topics concurrently.

For each topic, the application expects to find a corresponding environment variable `INPUT_TOPIC_CONFIG_{id}`,
for example `INPUT_TOPIC_CONFIG_1` or `INPUT_TOPIC_CONFIG_some-name`.
The value of this variable is assumed to be a JSON object containing Kafka topic configuration properties.

The JSON config has two fields:
1. `streams` - boolean flag indicating whether to use Kafka Streams API or not
2. `properties` - map of Kafka client configuration properties

For example:

```json
{
  "streams": true,
  "properties": {
    "bootstrap.servers": "localhost:9092",
    "application.id": "my-app"
  }
}
```

```json
{
  "streams": false,
  "properties": {
    "bootstrap.servers": "localhost:9092",
    "group.id": "my-group"
  }
}
```

The application will set some implicit properties:
1. For Kafka Streams API:
   * `processing.guarantee=exactly_once_v2`
2. For standard API:
   * `enable.auto.commit=false`
   * `isolation.level=read_committed`
   * `key.deserializer=org.apache.kafka.common.serialization.StringDeserializer`
   * `value.deserializer=org.apache.kafka.common.serialization.ByteArrayDeserializer`

# Consumption

Every received record is printed to STDOUT in a JSON log, with the following fields:
1. `key` - record key
2. `topic` - origin topic
3. `message` - `Received record`

These logs can be used to check if the app is consuming correct records.
The app commits offsets after each successful record consumption.
