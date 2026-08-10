use std::{
    collections::{BTreeMap, VecDeque},
    pin::pin,
    sync::Arc,
    time::Duration,
};

use anyhow::{Context, Result, anyhow};
use rdkafka::{
    ClientConfig, Offset, TopicPartitionList,
    consumer::{BaseConsumer, CommitMode, Consumer, StreamConsumer},
    message::{Headers, Message, OwnedMessage},
};
use serde::Serialize;
use tokio::{
    sync::{mpsc, oneshot},
    task,
};

#[derive(Serialize)]
pub struct ConsumedMessage {
    topic: String,
    partition: i32,
    offset: i64,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<String>,
}

enum Command {
    Consume {
        count: usize,
        wait: Duration,
        reply_to: oneshot::Sender<Result<Vec<ConsumedMessage>>>,
    },
    Peek {
        topic: String,
        count: usize,
        wait: Duration,
        reply_to: oneshot::Sender<Result<Vec<ConsumedMessage>>>,
    },
    Shutdown {
        reply_to: oneshot::Sender<()>,
    },
}

pub struct App {
    consumer: StreamConsumer,
    address: Arc<str>,
    topic: Arc<str>,
    /// Background-polled messages that `/consume` has not committed yet.
    buffer: VecDeque<OwnedMessage>,
}

impl App {
    /// Returns the command handle and the actor task so the caller can wait for the consumer to
    /// leave its group before exiting the process.
    pub fn spawn(address: &str, group: &str, topic: &str) -> anyhow::Result<AppHandle> {
        let app = Self::new(address, group, topic)?;
        let (tx, rx) = mpsc::channel(16);
        _ = tokio::spawn(app.enter_loop(rx));
        Ok(AppHandle { tx })
    }

    fn new(address: &str, group: &str, topic: &str) -> anyhow::Result<Self> {
        let consumer: StreamConsumer = ClientConfig::new()
            .set("bootstrap.servers", address)
            .set("group.id", group)
            // Keep lag open until `/consume` explicitly commits.
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            // Drain tests restart this pod via patching, so evict dead group members quickly.
            .set("session.timeout.ms", "6000")
            .set("heartbeat.interval.ms", "2000")
            .set("fetch.message.max.bytes", "10485760")
            .set("message.max.bytes", "10485760")
            .create()
            .context("failed to create consumer")?;

        consumer
            .subscribe(&[topic])
            .context("failed to subscribe to topic")?;

        Ok(Self {
            consumer,
            address: Arc::from(address),
            topic: Arc::from(topic),
            buffer: VecDeque::new(),
        })
    }

    async fn enter_loop(mut self, mut rx: mpsc::Receiver<Command>) -> Result<()> {
        loop {
            tokio::select! {
                msg = rx.recv() => {
                    let Some(msg) = msg else {
                        anyhow::bail!("command channel closed");
                    };
                    match msg {
                        Command::Consume { count, wait, reply_to } => {
                            let _ = reply_to.send(self.consume(count, wait).await);
                        }
                        Command::Peek { topic, count, wait, reply_to } => {
                            let _ = reply_to.send(self.peek(topic, count, wait).await);
                        }
                        Command::Shutdown { reply_to } => {
                            drop(self);
                            let _ = reply_to.send(());
                            return Ok(())
                        }
                    }
                },

                result = self.consumer.recv() => match result {
                    Ok(message) => self.buffer.push_back(message.detach()),
                    Err(error) => {
                        tracing::warn!(%error, "background poll error");
                        tokio::time::sleep(Duration::from_millis(250)).await;
                    }
                },
            }
        }
    }

    async fn consume(&mut self, count: usize, wait: Duration) -> Result<Vec<ConsumedMessage>> {
        let mut messages = Vec::new();

        while messages.len() < count {
            match self.buffer.pop_front() {
                Some(message) => messages.push(message),
                None => break,
            }
        }

        if messages.len() < count {
            let mut timeout = pin!(tokio::time::sleep(wait));
            while messages.len() < count {
                tokio::select! {
                    () = timeout.as_mut() => break,
                    msg = self.consumer.recv() => match msg {
                        Ok(message) => messages.push(message.detach()),
                        Err(error) => return Err(anyhow!("failed polling during consume: {error}")),
                    }
                }
            }
        }

        self.commit_with_retry(&messages)
            .await
            .context("failed to commit consumed offsets")?;
        Ok(messages.iter().map(Self::to_consumed).collect())
    }

    /// Reads a deterministic snapshot without touching the target consumer group's committed
    /// offsets.
    async fn peek(
        &self,
        topic: String,
        count: usize,
        wait: Duration,
    ) -> Result<Vec<ConsumedMessage>> {
        let address = self.address.clone();
        // Snapshot reads use a throwaway `BaseConsumer` with manual assignment so they stay
        // deterministic and never touch the target group's offsets; that API is sync, so keep it
        // off the actor runtime.
        task::spawn_blocking(move || Self::read_topic_snapshot(&address, &topic, count, wait))
            .await
            .context("peek task panicked")?
    }

    fn read_topic_snapshot(
        address: &str,
        topic: &str,
        count: usize,
        wait: Duration,
    ) -> Result<Vec<ConsumedMessage>> {
        let group = format!(
            "peek-observer-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        );
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", address)
            .set("group.id", group)
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("fetch.message.max.bytes", "10485760")
            .create()
            .context("failed to create peek consumer")?;

        let metadata = consumer
            .fetch_metadata(Some(topic), Duration::from_secs(10))
            .context("failed to fetch metadata for peek")?;
        let mut assignment = TopicPartitionList::new();
        for topic_metadata in metadata.topics() {
            for partition in topic_metadata.partitions() {
                assignment.add_partition_offset(topic, partition.id(), Offset::Beginning)?;
            }
        }
        consumer
            .assign(&assignment)
            .context("failed to assign partitions for peek")?;

        let mut messages = Vec::new();
        let deadline = std::time::Instant::now() + wait;
        loop {
            if messages.len() >= count {
                break;
            }

            let now = std::time::Instant::now();
            if now >= deadline {
                break;
            }

            match consumer.poll(Duration::from_millis(250).min(deadline - now)) {
                Some(Ok(message)) => messages.push(message.detach()),
                Some(Err(error)) => tracing::warn!(%error, "peek poll error"),
                None => {}
            }
        }
        Ok(messages.iter().map(Self::to_consumed).collect())
    }

    /// Commits the highest consumed offset per partition.
    fn commit(&self, messages: &[OwnedMessage]) -> Result<(), rdkafka::error::KafkaError> {
        if messages.is_empty() {
            return Ok(());
        }

        let Some(offsets) = Self::commit_offsets(messages) else {
            return Ok(());
        };

        let mut tpl = TopicPartitionList::new();
        for (partition, offset) in offsets {
            tpl.add_partition_offset(&self.topic, partition, Offset::Offset(offset))?;
        }

        self.consumer.commit(&tpl, CommitMode::Sync)
    }

    async fn commit_with_retry(&mut self, messages: &[OwnedMessage]) -> Result<()> {
        let mut timeout = pin!(tokio::time::sleep(Duration::from_secs(15)));
        let mut interval = pin!(tokio::time::interval(Duration::from_millis(250)));
        let mut last_error = anyhow!("commit with retry timeout reached");
        loop {
            tokio::select! {
                () = timeout.as_mut() => return Err(last_error),
                _ = interval.tick() => {
                    match self.commit(messages) {
                        Ok(()) => return Ok(()),
                        Err(error) => {
                            tracing::warn!(%error, "commit failed during consume, retrying");
                            last_error = error.into();
                            last_error = last_error.context("commit with retry");
                        }
                    }
                }
            }
        }
    }

    fn commit_offsets(messages: &[OwnedMessage]) -> Option<BTreeMap<i32, i64>> {
        let mut offsets: BTreeMap<i32, i64> = BTreeMap::new();
        for message in messages {
            offsets
                .entry(message.partition())
                .and_modify(|offset| *offset = (*offset).max(message.offset() + 1))
                .or_insert(message.offset() + 1);
        }

        (!offsets.is_empty()).then_some(offsets)
    }

    fn to_consumed(message: &OwnedMessage) -> ConsumedMessage {
        ConsumedMessage {
            topic: message.topic().to_owned(),
            partition: message.partition(),
            offset: message.offset(),
            headers: message
                .headers()
                .map(|headers| {
                    headers
                        .iter()
                        .filter_map(|header| {
                            Some((
                                header.key.to_owned(),
                                String::from_utf8(header.value?.to_vec()).ok()?,
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default(),
            payload: message
                .payload_view::<str>()
                .and_then(Result::ok)
                .map(str::to_owned),
        }
    }
}

#[derive(Clone)]
pub struct AppHandle {
    tx: mpsc::Sender<Command>,
}

impl AppHandle {
    async fn call<R>(&self, f: impl FnOnce(oneshot::Sender<R>) -> Command) -> anyhow::Result<R> {
        let (reply_to, rx) = oneshot::channel();
        self.tx
            .send(f(reply_to))
            .await
            .map_err(|_| anyhow!("app actor down"))?;
        rx.await.map_err(|_| anyhow!("app actor dropped reply_to"))
    }

    pub async fn consume(
        &self,
        count: usize,
        wait: Duration,
    ) -> anyhow::Result<Vec<ConsumedMessage>> {
        self.call(|reply_to| Command::Consume {
            count,
            wait,
            reply_to,
        })
        .await?
    }

    pub async fn peek(
        &self,
        topic: String,
        count: usize,
        wait: Duration,
    ) -> anyhow::Result<Vec<ConsumedMessage>> {
        self.call(|reply_to| Command::Peek {
            topic,
            count,
            wait,
            reply_to,
        })
        .await?
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.call(|reply_to| Command::Shutdown { reply_to }).await
    }
}
