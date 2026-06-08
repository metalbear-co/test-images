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
    val headers: Map<String, String> = emptyMap(),
    val value: String = "",
    // Target size in bytes for the produced value. Used to test large payloads that cannot be
    // passed literally through an env var, since a single env var string is capped at 128 KiB.
    // When set, the value is repeated/padded up to this many bytes before being sent.
    val valueSize: Int? = null,
) {
    override fun toString(): String = Json.encodeToString(this)
}
