// Kafka test consumer built on KafkaJS.
//
// Exists to exercise consumers whose group-protocol name differs from
// librdkafka's: KafkaJS advertises `RoundRobinAssigner` when joining its
// consumer group, which librdkafka-based clients cannot negotiate with.
//
// Mirrors the kafka-consumer (Java) image's contract so test harnesses can use
// either image interchangeably:
// - env: `KAFKA_<property>` client properties (`KAFKA_bootstrap.servers`,
//   `KAFKA_group.id`), `INPUT_TOPIC_*` subscriptions
// - logs: one JSON object per line; `configuration` on startup, `value` per
//   received record, and a message starting with "Received shutdown signal"
//   when terminating.
const { Kafka, logLevel } = require("kafkajs");

function log(fields) {
  console.log(JSON.stringify(fields));
}

function readConfiguration() {
  const properties = {};
  const inputTopics = [];
  for (const [key, value] of Object.entries(process.env)) {
    if (key.startsWith("KAFKA_")) {
      properties[key.slice("KAFKA_".length)] = value;
    }
    if (key.startsWith("INPUT_TOPIC_")) {
      inputTopics.push(value);
    }
  }
  return { properties, inputTopics };
}

const config = readConfiguration();
const brokers = (config.properties["bootstrap.servers"] || "").split(",");
const groupId = config.properties["group.id"];

const kafka = new Kafka({
  clientId: "kafkajs-consumer",
  brokers,
  logLevel: logLevel.NOTHING,
});

let consumer = null;
let stopping = false;

async function runOnce() {
  consumer = kafka.consumer({ groupId });
  await consumer.connect();
  for (const topic of config.inputTopics) {
    // `fromBeginning` matches the Java image's `auto.offset.reset=earliest`:
    // a group with no committed offsets starts from the oldest message.
    await consumer.subscribe({ topic, fromBeginning: true });
  }

  await consumer.run({
    eachMessage: async ({ topic, message }) => {
      log({
        message: "Received record",
        key: message.key ? message.key.toString() : null,
        topic,
        value: message.value ? message.value.toString() : null,
      });
    },
  });
}

// Topics can be created asynchronously (Strimzi reconciles KafkaTopic CRs
// after this pod starts), so retry instead of exiting: a crash-looping pod
// never turns Ready.
async function main() {
  log({ message: "Fetched configuration from env", configuration: config });
  if (!groupId) {
    throw new Error("Expected 'group.id' in properties (KAFKA_group.id env var)");
  }

  while (!stopping) {
    try {
      await runOnce();
      return;
    } catch (error) {
      log({ message: "Consumer failed, retrying in 5s", error: String(error) });
      try {
        await consumer?.disconnect();
      } catch {}
      await new Promise((resolve) => setTimeout(resolve, 5000));
    }
  }
}

for (const signal of ["SIGTERM", "SIGINT"]) {
  process.once(signal, async () => {
    stopping = true;
    log({ message: "Received shutdown signal, closing consumer" });
    try {
      await consumer?.disconnect();
    } finally {
      process.exit(0);
    }
  });
}

main().catch((error) => {
  log({ message: "Consumer failed", error: String(error) });
  process.exit(1);
});
