#![deny(unused_crate_dependencies)]

use core::panic;
use std::collections::HashMap;
use std::fmt;
use std::os::unix::ffi::OsStrExt;

use aws_sdk_sqs::types::MessageSystemAttributeName;
use aws_sdk_sqs::{types::Message, Client};
use tokio::task::JoinSet;
use tokio::time::{sleep, Duration};

use crate::config::{ForwardingConfig, SqsQueueEnv};

mod config;

struct Forwarder {
    client: Client,
    config: ForwardingConfig,
    resolved_input: String,
    resolved_output: String,
}

impl Forwarder {
    async fn new(client: Client, config: ForwardingConfig) -> Self {
        let resolved_input = Self::resolve_queue_url(&client, &config.from).await;
        let resolved_output = Self::resolve_queue_url(&client, &config.to).await;

        println!(
            "[{}:{}] [{config:?}] Resolved queue URLs: {resolved_input} -> {resolved_output}",
            file!(),
            line!(),
        );

        Self {
            client,
            config,
            resolved_input,
            resolved_output,
        }
    }

    async fn resolve_queue_url(client: &Client, queue_env: &SqsQueueEnv) -> String {
        let SqsQueueEnv {
            var_name,
            is_url,
            json_key,
        } = queue_env;

        let value = std::env::var_os(var_name).unwrap_or_else(|| {
            panic!(
                "[{}:{}] Environment variable `{var_name}` is not set, queue_env=[{queue_env:?}]",
                file!(),
                line!()
            );
        });

        let value = match json_key {
            Some(key) => {
                serde_json::from_slice::<HashMap<&str, &str>>(value.as_bytes()).unwrap_or_else(|error| {
                    panic!(
                        "[{}:{}] [{queue_env:?}] Environment variable `{var_name}` is not a valid JSON object: {error}",
                        file!(),
                        line!(),
                    );
                }).remove(key.as_str()).unwrap_or_else(|| {
                    panic!(
                        "[{}:{}] [{queue_env:?}] JSON object read from environment variable `{var_name}` does not contain the key `{key}`",
                        file!(),
                        line!()
                    );
                })
            }
            None => value.to_str().unwrap_or_else(|| {
                panic!(
                    "[{}:{}] [{queue_env:?}] Environment variable `{var_name}` is not a valid UTF-8 string, env_value=[{value:?}]",
                    file!(),
                    line!()
                )
            }),
        };

        if *is_url {
            return value.to_string();
        }

        client
            .get_queue_url()
            .queue_name(value)
            .send()
            .await
            .unwrap_or_else(|error| {
                panic!(
                    "[{}:{}] [{queue_env:?}] Failed to resolve URL of SQS queue named `{value}`: {error}",
                    file!(),
                    line!(),
                );
            })
            .queue_url
            .unwrap_or_else(|| {
                panic!(
                    "[{}:{}] [{queue_env:?}] Failed to resolve URL of SQS queue named `{value}`: no queue url in the response",
                    file!(),
                    line!(),
                );
            })
    }

    async fn run(&self) {
        let receive_message_request = self
            .client
            .receive_message()
            .message_attribute_names(".*")
            .message_system_attribute_names(MessageSystemAttributeName::All)
            .wait_time_seconds(20)
            .max_number_of_messages(1)
            .queue_url(&self.resolved_input);

        loop {
            let messages = match receive_message_request.clone().send().await {
                Ok(response) => response.messages.unwrap_or_default(),
                Err(error) => {
                    println!(
                        "[{}:{}] [{self:?}] Failed to receive messages: {error}",
                        file!(),
                        line!()
                    );
                    sleep(Duration::from_secs(3)).await;
                    continue;
                }
            };

            for message in messages {
                println!(
                    "[{}:{}] [{self:?}] Received message: id=[{:?}], attributes=[{:?}], body=[{:?}]",
                    file!(),
                    line!(),
                    message.message_id,
                    message.message_attributes,
                    message.body,
                );

                self.pass_forward(message).await;
            }
        }
    }

    async fn pass_forward(&self, mut message: Message) {
        let group_id = message
            .attributes
            .as_mut()
            .and_then(|attr_map| attr_map.remove(&MessageSystemAttributeName::MessageGroupId));

        let deduplication_id = message.attributes.and_then(|mut attr_map| {
            attr_map.remove(&MessageSystemAttributeName::MessageDeduplicationId)
        });

        let send_result = self
            .client
            .send_message()
            .queue_url(&self.resolved_output)
            .set_message_attributes(message.message_attributes)
            .set_message_group_id(group_id)
            .set_message_deduplication_id(deduplication_id)
            .set_message_body(message.body)
            .send()
            .await;

        match send_result {
            Ok(..) => {
                println!(
                    "[{}:{}] [{self:?}] Successfully forwarded the message",
                    file!(),
                    line!(),
                );
            }
            Err(error) => {
                panic!(
                    "[{}:{}] [{self:?}] Failed to forward the message: {error}",
                    file!(),
                    line!(),
                );
            }
        }

        let Some(handle) = message.receipt_handle else {
            return;
        };

        let delete_result = self
            .client
            .delete_message()
            .queue_url(&self.resolved_input)
            .receipt_handle(handle)
            .send()
            .await;
        match delete_result {
            Ok(..) => {
                println!(
                    "[{}:{}] [{self:?}] Successfully deleted the message from the input queue.",
                    file!(),
                    line!(),
                );
            }
            Err(error) => {
                panic!(
                    "[{}:{}] [{self:?}] Failed to delete the message from the input queue: {error}",
                    file!(),
                    line!(),
                );
            }
        }
    }
}

impl fmt::Debug for Forwarder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Forwarder")
            .field("config", &self.config)
            .field("input", &self.resolved_input)
            .field("output", &self.resolved_output)
            .finish()
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let sdk_config = aws_config::load_from_env().await;
    let client = Client::new(&sdk_config);
    let config = config::resolve_config();

    let mut tasks = JoinSet::new();
    for config in config {
        let client = client.clone();
        tasks.spawn(async move {
            let forwarder = Forwarder::new(client, config).await;
            forwarder.run().await;
        });
    }

    tasks.join_all().await;
}
