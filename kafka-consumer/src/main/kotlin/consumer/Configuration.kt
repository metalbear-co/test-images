package com.metalbear.consumer

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

/**
 * Configuration for the consumer app.
 */
@Serializable
data class Configuration(
    val properties: Map<String, String>,
    val inputTopics: List<String>,
) {
    companion object {
        fun readFromEnv(): Configuration {
            val env = System.getenv()
            val properties =
                env.filterKeys { it.startsWith("KAFKA_") }.mapKeys { it.key.removePrefix("KAFKA_") }
            val topics =
                env
                    .filterKeys { it.startsWith("INPUT_TOPIC_") }
                    .values
                    .toList()
            return Configuration(properties, topics)
        }
    }

    override fun toString(): String = Json.encodeToString(this)
}
