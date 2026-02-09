package com.metalbear

import kotlinx.serialization.Serializable

@Serializable
data class TopicConfig(
    val streams: Boolean,
    val properties: Map<String, String>,
)
