package com.metalbear

import com.metalbear.producer.Configuration
import com.metalbear.producer.OutputTopic
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.isActive
import kotlinx.coroutines.joinAll
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import net.logstash.logback.argument.StructuredArguments.keyValue
import org.apache.kafka.clients.admin.AdminClient
import org.apache.kafka.clients.admin.NewTopic
import org.apache.kafka.clients.producer.KafkaProducer
import org.apache.kafka.clients.producer.ProducerRecord
import org.slf4j.Logger
import org.slf4j.LoggerFactory
import java.util.Properties

fun main() =
    runBlocking {
        val logger = LoggerFactory.getLogger(Logger.ROOT_LOGGER_NAME)

        val config = Configuration.readFromEnv()
        logger.info(
            "Fetched configuration from env",
            keyValue("configuration", config),
        )

        val properties =
            Properties().apply {
                putAll(config.properties)
            }
        createTopics(properties, config.outputTopics.map { it.name })

        val producer = KafkaProducer<String, String>(properties)
        val job = SupervisorJob()
        val jobScope = CoroutineScope(job)
        val jobs =
            config.outputTopics.map { topic ->
                jobScope.launch {
                    produceMessages(producer, topic)
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

fun createTopics(
    properties: Properties,
    topics: List<String>,
) {
    val client = AdminClient.create(properties)
    val existing = client.listTopics().names().get()
    val toCreate = topics.filter { it !in existing }.map { NewTopic(it, 1, 1) }
    client.createTopics(toCreate).all().get()
}

suspend fun produceMessages(
    producer: KafkaProducer<String, String>,
    topic: OutputTopic,
) {
    val logger = LoggerFactory.getLogger("producer-${topic.name}")

    for (message in topic.messages) {
        if (currentCoroutineContext().isActive.not()) {
            break
        }

        val record = ProducerRecord(topic.name, message.key, "hello")
        message.headers.forEach { record.headers().add(it.key, it.value.toByteArray()) }

        val metadata =
            withContext(Dispatchers.IO + NonCancellable) {
                producer.send(record).get()
            }
        logger.info(
            "Produced record",
            keyValue("topic", topic.name),
            keyValue("partition", metadata.partition()),
            keyValue("offset", metadata.offset()),
        )
    }

    withContext(Dispatchers.IO + NonCancellable) {
        producer.close()
    }
}
