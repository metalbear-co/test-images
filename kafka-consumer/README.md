Simple app that consumes or produces records from/to multiple Kafka topics.

Consumption can be done with Kafka Streams API or with the standard KafkaConsumer.

# Consumer

Consumer entrypoint is `com.metalbear.ConsumerKt`.

## Configuration

Consumer is configured like this:

1. All environment variables prefixed with `KAFKA_` are interpreted as Kafka client properties.
2. Values of all environment variables prefixed with `INPUT_TOPIC_` are interpreted as input Kafka topics to read from.
3. Resolved client properties must contain either the `group.id` or `application.id` property.
   If `group.id` is present, the app will use KafkaConsumer API.
   If `application.id` is present, the app will use KafkaStreams API.

Example configuration:

1. `KAFKA_bootrstrap.servers=localhost:9092`
2. `KAFKA_application.id=my-group`
3. `INPUT_TOPIC_1=my-topic-1`
4. `INPUT_TOPIC_2=my-topic-2`

For consumption with standard KafkaConsumer API, the app automatically sets:

1. `key.deserializer=org.apache.kafka.common.serialization.StringDeserializer`
2. `value.deserializer=org.apache.kafka.common.serialization.StringDeserializer`
3. `enable.auto.commit=false`

For consumption with KafkaStreams API, the app automatically sets:

1. `processing.guarantee=exactly_once_v2`

## Consumption

Consumed records are logged to STDOUT in a JSON format, and discarded.
Each such log contains following fields:

1. `message` - `Received record`
2. `topic` - origin topic
3. `key` - record key
4. `value` - record value

# Producer

Produced entrypoint is `com.metalbear.ProducerKt`.

## Configuration

Producer is configured like this:

1. All environment variables prefixed with `KAFKA_` are interpreted as Kafka client properties.
2. Values of all environment variables prefixed with `OUTPUT_TOPIC_` are parsed into JSON lists of messages to produce,
   and the remainder of the variable key is interpreted as the topic name.

Example configuration:

1. `KAFKA_bootstrap.servers=localhost:9092`
2. `OUTPUT_TOPIC_topic-name-1=[{"key": "key-1", "headers": {"header-1": "value-1"}, "value": "value-1"}]`

As this app can only produce messages with String keys and values,
`key.serializer` and `key.deserializer` properties are set automatically.

## Production

Messages are produced in the given order. If the topics don't exist, they will be created automatically.
