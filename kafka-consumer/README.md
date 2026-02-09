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
3. `KAFKA_processing.guarantee=exactly_once_v2`
4. `INPUT_TOPIC_1=my-topic-1`
5. `INPUT_TOPIC_2=my-topic-2`

## Consumption

Consumed records are logged to STDOUT in a JSON format, and discarded.
Each such log contains following fields:
1. `message` - `Received record`
2. `topic` - origin topic
3. `key` - record key

# Producer

Produced entrypoint is `com.metalbear.ProducerKt`.

## Configuration

Producer is configured like this:
1. All environment variables prefixed with `KAFKA_` are interpreted as Kafka client properties.
2. Values of all environment variables prefixed with `OUTPUT_TOPIC_` are parsed into JSON lists of messages to produce,
and the remainder of the variable key is interpreted as the topic name.

Example configuration:
1. `KAFKA_bootstrap.servers=localhost:9092`
2. `KAFKA_key.serializer=org.apache.kafka.common.serialization.StringSerializer`
3. `KAFKA_value.serializer=org.apache.kafka.common.serialization.StringSerializer`
4. `OUTPUT_TOPIC_topic-name-1=[{"key": "key-1", "headers": {"header-1": "value-1"}}]`

## Production

Messages are produced in the given order, with given keys/headers, and bodies hardcoded to `hello.`

If the topics don't exist, they will be created automatically.
