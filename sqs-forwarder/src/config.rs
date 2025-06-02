use std::os::unix::ffi::OsStrExt;

use serde::Deserialize;

/// Name of the environment variable that holds the configuration for this app.
///
/// The configuration is expected to be a JSON array of [`SqsQueueEnv`] objects.
/// If the variable is not set, the app will use [`legacy_config`].
/// This is to maintain backwards compatibility with mirrord Operator E2E.
pub const CONFIGURATION_ENV_NAME: &str = "SQS_FORWARDER_CONFIG";

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForwardingConfig {
    /// Queue from which we should read messages.
    pub from: SqsQueueEnv,
    /// Queue to which we should echo messages.
    pub to: SqsQueueEnv,
}

/// Describes a source of SQS queue name/URL for this app.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SqsQueueEnv {
    /// Name of the environment variable that holds the SQS queue name or URL.
    pub var_name: String,
    /// Whether the environment variable holds a URL or a queue name.
    #[serde(default)]
    pub is_url: bool,
    /// If set, the value of the environment variable will be parsed into a JSON object.
    /// The queue name/URL will be extracted from the field with this name.
    pub json_key: Option<String>,
}

fn legacy_config() -> Vec<ForwardingConfig> {
    vec![
        ForwardingConfig {
            from: SqsQueueEnv {
                var_name: "SQS_TEST_Q_NAME1".into(),
                is_url: false,
                json_key: None,
            },
            to: SqsQueueEnv {
                var_name: "SQS_TEST_ECHO_Q_NAME1".into(),
                is_url: false,
                json_key: None,
            },
        },
        ForwardingConfig {
            from: SqsQueueEnv {
                var_name: "SQS_TEST_Q2_URL".into(),
                is_url: true,
                json_key: None,
            },
            to: SqsQueueEnv {
                var_name: "SQS_TEST_ECHO_Q_NAME2".into(),
                is_url: false,
                json_key: None,
            },
        },
    ]
}

/// If [`CONFIGURATION_ENV_NAME`] is set, this function will parse it into a JSON array of [`ForwardingConfig`].
///
/// Otherwise, it will return the legacy configuration:
/// 1. queue name from `SQS_TEST_Q_NAME1` -> queue name from `SQS_TEST_ECHO_Q_NAME1`
/// 2. queue URL from `SQS_TEST_Q2_URL` -> queue name from `SQS_TEST_ECHO_Q_NAME2`.
pub fn resolve_config() -> Vec<ForwardingConfig> {
    std::env::var_os(CONFIGURATION_ENV_NAME)
        .map(|config_raw| {
            serde_json::from_slice(config_raw.as_bytes())
                .expect("failed to parse SQS_FORWARDER_CONFIG")
        })
        .unwrap_or_else(legacy_config)
}
