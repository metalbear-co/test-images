package com.metalbear

import com.metalbear.consumer.Configuration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.cancel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import net.logstash.logback.argument.StructuredArguments.keyValue
import org.apache.kafka.clients.consumer.ConsumerConfig
import org.apache.kafka.clients.consumer.KafkaConsumer
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

        val config = Configuration.readFromEnv()
        logger.info(
            "Fetched configuration from env",
            keyValue("configuration", config),
        )

        val streams =
            if (config.properties.containsKey("group.id")) {
                false
            } else if (config.properties.containsKey("application.id")) {
                true
            } else {
                throw RuntimeException("Expected either 'group.id' or 'application.id' in properties, found none")
            }
        val properties = Properties().apply { putAll(config.properties) }

        val jobScope = CoroutineScope(SupervisorJob())
        val job =
            jobScope.launch {
                if (streams) {
                    properties[StreamsConfig.PROCESSING_GUARANTEE_CONFIG] = StreamsConfig.EXACTLY_ONCE_V2
                    runStreamsConsumer(properties, config.inputTopics)
                } else {
                    properties[ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG] = StringDeserializer::class.java.name
                    properties[ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG] = StringDeserializer::class.java.name
                    properties[ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG] = false
                    runStandardConsumer(properties, config.inputTopics)
                }
            }

        Runtime.getRuntime().addShutdownHook(
            Thread {
                logger.info("Received shutdown signal, cancelling jobs")
                jobScope.cancel()
                runBlocking { job.join() }
            },
        )

        job.join()
    }

/**
 * Consumes records from the given topics, using standard KafkaConsumer
 */
suspend fun runStandardConsumer(
    properties: Properties,
    topics: List<String>,
) {
    val logger = LoggerFactory.getLogger("standard-consumer-task")

    val consumer =
        withContext(Dispatchers.IO) {
            KafkaConsumer<String, String>(properties).apply {
                subscribe(topics)
            }
        }

    while (currentCoroutineContext().isActive) {
        val records =
            withContext(Dispatchers.IO + NonCancellable) {
                consumer.poll(Duration.ofMillis(100))
            }

        records.forEach {
            logger.info(
                "Received record",
                keyValue("key", it.key()),
                keyValue("topic", it.topic()),
                keyValue("value", it.value()),
            )
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
    logger.info("Closed consumer")
    return
}

/**
 * Consumes records from the given topic, using standard Kafka Streams API.
 */
suspend fun runStreamsConsumer(
    properties: Properties,
    topics: List<String>,
) {
    val logger = LoggerFactory.getLogger("streams-consumer-task")

    val builder = StreamsBuilder()
    topics.forEach { topic ->
        val stream = builder.stream(topic, Consumed.with(Serdes.String(), Serdes.String()))
        stream.foreach { key, value ->
            logger.info(
                "Received record",
                keyValue("key", key),
                keyValue("topic", topic),
                keyValue("value", value),
            )
        }
    }

    val topology = builder.build()
    val kafkaStreams = KafkaStreams(topology, properties)

    kafkaStreams.start()

    try {
        awaitCancellation()
    } catch (_: CancellationException) {
        logger.info("Received shutdown signal")
        withContext(Dispatchers.IO + NonCancellable) {
            kafkaStreams.close()
        }
        logger.info("Closed streams")
    }
}
