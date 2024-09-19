# sqs-forwarder

Test application to be deployed to the cluster in mirrord E2E tests.

This application reads from 2 queues, and forwards read messages to 2 "echo"
queues. That way, E2E tests can run SQS splitting, write messages to the 2 input
queues, and read from the "echo" queues, to make sure this deployed application
got exactly the messages it was supposed to get.

This forward-to-other queue mechanism was chosen because the test has to use the
SQS API anyway, so it's convenient to also use that to verify the messages that
arrived at the deployed app (and not e.g. printing the messages in the deployed
app and fetching the logs in the test with the K8s API).