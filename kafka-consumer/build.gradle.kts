plugins {
    kotlin("jvm") version "2.2.20"
    kotlin("plugin.serialization") version "2.2.20"
    id("org.jlleitschuh.gradle.ktlint") version "14.0.1"
    application
}

group = "com.metalbear"
version = "0.1-SNAPSHOT"

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.8.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.10.2")
    implementation("org.apache.kafka:kafka-clients:3.9.1")
    implementation("org.apache.kafka:kafka-streams:3.9.0")
    implementation("ch.qos.logback:logback-classic:1.5.28")
    implementation("net.logstash.logback:logstash-logback-encoder:8.0")
    testImplementation(kotlin("test"))
}

tasks.test {
    useJUnitPlatform()
}

kotlin {
    jvmToolchain(21)
}

application {
    mainClass.set("com.metalbear.ConsumerKt")
}

tasks.jar {
    manifest {
        attributes["Main-Class"] = "com.metalbear.ConsumerKt"
    }
    from(configurations.runtimeClasspath.get().map { if (it.isDirectory) it else zipTree(it) })
    duplicatesStrategy = DuplicatesStrategy.EXCLUDE
}
