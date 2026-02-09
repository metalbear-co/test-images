package com.metalbear

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.cancel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.isActive
import kotlinx.coroutines.joinAll
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import net.logstash.logback.argument.StructuredArguments.keyValue
import org.apache.kafka.clients.consumer.ConsumerConfig
import org.apache.kafka.clients.consumer.KafkaConsumer
import org.apache.kafka.common.serialization.ByteArrayDeserializer
import org.apache.kafka.common.serialization.Serdes
import org.apache.kafka.common.serialization.StringDeserializer
import org.apache.kafka.streams.KafkaStreams
import org.apache.kafka.streams.StreamsBuilder
import org.apache.kafka.streams.StreamsConfig
import org.apache.kafka.streams.kstream.Consumed
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import java.time.Duration
import java.util.Properties
import kotlin.coroutines.cancellation.CancellationException

fun main() =
    runBlocking {
        val logger = LoggerFactory.getLogger(Logger.ROOT_LOGGER_NAME)

        val env = System.getenv()
        val topics =
            env
                .filterKeys { it.startsWith("INPUT_TOPIC_NAME_") }
                .mapKeys { it.key.removePrefix("INPUT_TOPIC_NAME_") }
                .mapValues {
                    val envName = "INPUT_TOPIC_CONFIG_${it.key}"
                    val raw = env[envName] ?: throw RuntimeException("Env variable $envName not set")
                    val config = Json.decodeFromString<TopicConfig>(raw)
                    Pair(it.value, config)
                }
        logger.info(
            "Fetched input topics from env",
            keyValue(
                "topics",
                topics.values.map {
                    mapOf(
                        "name" to it.first,
                        "streams" to it.second.streams,
                        "properties" to it.second.properties,
                    )
                },
            ),
        )

        val job = SupervisorJob()
        val jobScope = CoroutineScope(job)
        val jobs =
            topics.values.map { (topic, config) ->
                jobScope.launch {
                    if (config.streams) {
                        runStreamsConsumer(topic, config.properties)
                    } else {
                        runStandardConsumer(topic, config.properties)
                    }
                }
            }

        Runtime.getRuntime().addShutdownHook(
            Thread {
                logger.info("Received shutdown signal, cancelling jobs")
                jobScope.cancel()
                runBlocking { jobs.joinAll() }
            },
        )

        job.join()
    }

/**
 * Consumes records from the given topic, using standard Kafka Consumer API.
 */
suspend fun runStandardConsumer(
    topic: String,
    properties: Map<String, String>,
) {
    val logger = LoggerFactory.getLogger("standard-consumer-$topic")

    val properties =
        Properties().apply {
            put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer::class.java.name)
            put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, ByteArrayDeserializer::class.java.name)
            put(ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, "false")
            put(ConsumerConfig.ISOLATION_LEVEL_CONFIG, "read_committed")
            putAll(properties)
        }
    val consumer =
        withContext(Dispatchers.IO) {
            KafkaConsumer<String, ByteArray>(properties).apply {
                subscribe(listOf(topic))
            }
        }

    while (currentCoroutineContext().isActive) {
        val records =
            withContext(Dispatchers.IO + NonCancellable) {
                consumer.poll(Duration.ofMillis(100))
            }

        records.forEach {
            logger.info("Received record", keyValue("key", it.key()), keyValue("topic", topic))
        }

        if (records.isEmpty.not()) {
            withContext(Dispatchers.IO + NonCancellable) {
                consumer.commitSync()
            }
        }
    }

    logger.info("Received shutdown signal")
    withContext(Dispatchers.IO + NonCancellable) {
        consumer.close()
    }
    return
}

/**
 * Consumes records from the given topic, using standard Kafka Streams API.
 */
suspend fun runStreamsConsumer(
    topic: String,
    properties: Map<String, String>,
) {
    val logger = LoggerFactory.getLogger("streams-consumer-$topic")

    val builder = StreamsBuilder()
    val stream = builder.stream(topic, Consumed.with(Serdes.String(), Serdes.ByteArray()))

    stream.foreach { key, _ ->
        logger.info("Received record", keyValue("key", key), keyValue("topic", topic))
    }

    val topology = builder.build()
    val properties =
        Properties().apply {
            put(StreamsConfig.PROCESSING_GUARANTEE_CONFIG, StreamsConfig.EXACTLY_ONCE_V2)
            putAll(properties)
        }
    val kafkaStreams = KafkaStreams(topology, properties)

    kafkaStreams.start()

    try {
        awaitCancellation()
    } catch (_: CancellationException) {
        logger.info("Received shutdown signal")
        withContext(Dispatchers.IO + NonCancellable) {
            kafkaStreams.close()
        }
    }
}
