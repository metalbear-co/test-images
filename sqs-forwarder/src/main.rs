use aws_sdk_sqs::types::MessageSystemAttributeName;
use aws_sdk_sqs::{operation::receive_message::ReceiveMessageOutput, types::Message, Client};
use tokio::time::{sleep, Duration};

/// Name of the environment variable that holds the name of the SQS queue to read from.
const QUEUE_NAME_ENV_VAR1: &str = "SQS_TEST_Q_NAME1";

/// Name of the environment variable that holds the name of the SQS queue to write to.
const ECHO_QUEUE_NAME_ENV_VAR1: &str = "SQS_TEST_ECHO_Q_NAME1";

/// Name of the environment variable that holds the url of the second SQS queue to read from.
/// Using URL, not name here, to test that functionality.
const QUEUE2_URL_ENV_VAR: &str = "SQS_TEST_Q2_URL";

/// Name of the environment variable that holds the name of the SQS queue to write to.
const ECHO_QUEUE_NAME_ENV_VAR2: &str = "SQS_TEST_ECHO_Q_NAME2";

/// Given Q name - read messages from that queue and echo them to the echo Q.
async fn read_from_queue_by_name(
    read_q_name: String,
    echo_q_name: String,
    client: Client,
) -> Result<(), anyhow::Error> {
    println!("Reading from Q: {read_q_name}, writing to echo Q: {echo_q_name}.");
    let read_q_url = client
        .get_queue_url()
        .queue_name(&read_q_name)
        .send()
        .await
        .inspect_err(|err| eprintln!("failed to get URL of queue {read_q_name}: {err:?}"))?
        .queue_url
        .unwrap();
    println!("Successfully fetched queue URL: {read_q_url}");
    read_from_queue_by_url(read_q_url, echo_q_name, client).await
}

async fn read_from_queue_by_url(
    read_q_url: String,
    echo_q_name: String,
    client: Client,
) -> Result<(), anyhow::Error> {
    let echo_q_url = client
        .get_queue_url()
        .queue_name(&echo_q_name)
        .send()
        .await
        .inspect_err(|err| eprintln!("failed to get URL of queue {echo_q_name}: {err:?}"))?
        .queue_url
        .unwrap();
    println!("Successfully fetched queue URL: {echo_q_url}");

    let receive_message_request = client
        .receive_message()
        .message_attribute_names(".*")
        .message_system_attribute_names(MessageSystemAttributeName::All)
        .wait_time_seconds(20)
        .queue_url(&read_q_url);
    loop {
        let res = match receive_message_request.clone().send().await {
            Ok(res) => res,
            Err(err) => {
                println!("ERROR: {err:?}");
                sleep(Duration::from_secs(3)).await;
                continue;
            }
        };
        if let ReceiveMessageOutput {
            messages: Some(messages),
            ..
        } = res
        {
            for Message {
                body,
                message_attributes,
                message_id,
                receipt_handle,
                mut attributes,
                ..
            } in messages
            {
                println!("message_id: {message_id:?}, message_attributes: {message_attributes:?}, body: {body:?}");
                let group_id = attributes.as_mut().and_then(|attr_map| {
                    attr_map.remove(&MessageSystemAttributeName::MessageGroupId)
                });

                let deduplication_id = attributes.and_then(|mut attr_map| {
                    attr_map.remove(&MessageSystemAttributeName::MessageDeduplicationId)
                });

                println!("forwarding message to {echo_q_name}");
                if let Err(err) = client
                    .send_message()
                    .queue_url(&echo_q_url)
                    .set_message_attributes(message_attributes)
                    .set_message_group_id(group_id)
                    .set_message_deduplication_id(deduplication_id)
                    .set_message_body(body)
                    .send()
                    .await
                {
                    println!("failed to forward message to output queue: {err:?}");
                } else if let Some(handle) = receipt_handle {
                    client
                        .delete_message()
                        .queue_url(&read_q_url)
                        .receipt_handle(handle)
                        .send()
                        .await
                        .inspect_err(|err| println!("deleting received message failed: {err:?}"))
                        .ok();
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let sdk_config = aws_config::load_from_env().await;
    let client = Client::new(&sdk_config);
    let read_q_name = std::env::var(QUEUE_NAME_ENV_VAR1).unwrap();
    let echo_queue_name = std::env::var(ECHO_QUEUE_NAME_ENV_VAR1).unwrap();
    let q_task_handle = tokio::spawn(read_from_queue_by_name(
        read_q_name,
        echo_queue_name,
        client.clone(),
    ));

    // using URL on second queue to test that too.
    let read_q_url = std::env::var(QUEUE2_URL_ENV_VAR).unwrap();
    let echo_queue_name = std::env::var(ECHO_QUEUE_NAME_ENV_VAR2).unwrap();
    let fifo_q_task_handle = tokio::spawn(read_from_queue_by_url(
        read_q_url,
        echo_queue_name,
        client.clone(),
    ));
    let (q_res, fifo_res) = tokio::join!(q_task_handle, fifo_q_task_handle);
    q_res.unwrap().unwrap();
    fifo_res.unwrap().unwrap();
}
