package com.metalbear.producer

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class Configuration(
    val properties: Map<String, String>,
    val outputTopics: List<OutputTopic>,
) {
    companion object {
        fun readFromEnv(): Configuration {
            val env = System.getenv()
            val properties =
                env.filterKeys { it.startsWith("KAFKA_") }.mapKeys { it.key.removePrefix("KAFKA_") }
            val topics =
                env
                    .filterKeys { it.startsWith("OUTPUT_TOPIC_") }
                    .mapKeys { it.key.removePrefix("OUTPUT_TOPIC_") }
                    .map {
                        val messages = Json.decodeFromString<List<OutputMessage>>(it.value)
                        OutputTopic(it.key, messages)
                    }
            return Configuration(properties, topics)
        }
    }

    override fun toString(): String = Json.encodeToString(this)
}

@Serializable
data class OutputTopic(
    val name: String,
    val messages: List<OutputMessage>,
) {
    override fun toString(): String = Json.encodeToString(this)
}

@Serializable
data class OutputMessage(
    val key: String,
    val headers: Map<String, String>,
) {
    override fun toString(): String = Json.encodeToString(this)
}
