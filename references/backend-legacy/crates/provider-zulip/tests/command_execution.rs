use std::sync::Mutex;

use makosh_provider_zulip::{
    client::{ZulipClientError, ZulipReactionRequest, ZulipUpdateMessageRequest},
    command_execution::{ZulipCommandTransport, ZulipExecutableCommand, execute_zulip_command},
    models::{ZulipBasicResponse, ZulipSendMessageResponse, ZulipUploadFileResponse},
};
use serde_json::json;

#[derive(Default)]
struct RecordingTransport {
    direct_recipients: Mutex<Vec<Vec<String>>>,
    direct_user_ids: Mutex<Vec<Vec<i64>>>,
}

#[async_trait::async_trait]
impl ZulipCommandTransport for RecordingTransport {
    async fn send_stream_message(
        &self,
        _: &str,
        _: &str,
        _: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        panic!("stream messages are not used by these tests")
    }
    async fn send_direct_message(
        &self,
        recipients: &[&str],
        _: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        self.direct_recipients
            .lock()
            .expect("direct recipient lock")
            .push(
                recipients
                    .iter()
                    .map(|recipient| (*recipient).to_owned())
                    .collect(),
            );
        Ok(ZulipSendMessageResponse {
            result: "success".to_owned(),
            msg: String::new(),
            id: Some(8801),
        })
    }
    async fn send_direct_message_to_user_ids(
        &self,
        recipient_user_ids: &[i64],
        _: &str,
    ) -> Result<ZulipSendMessageResponse, ZulipClientError> {
        self.direct_user_ids
            .lock()
            .expect("direct user id lock")
            .push(recipient_user_ids.to_vec());
        Ok(ZulipSendMessageResponse {
            result: "success".to_owned(),
            msg: String::new(),
            id: Some(9901),
        })
    }
    async fn update_message(
        &self,
        _: i64,
        _: &ZulipUpdateMessageRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        panic!("message updates are not used by these tests")
    }
    async fn delete_message(&self, _: i64) -> Result<ZulipBasicResponse, ZulipClientError> {
        panic!("message deletes are not used by these tests")
    }
    async fn add_reaction(
        &self,
        _: i64,
        _: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        panic!("reactions are not used by these tests")
    }
    async fn remove_reaction(
        &self,
        _: i64,
        _: &ZulipReactionRequest,
    ) -> Result<ZulipBasicResponse, ZulipClientError> {
        panic!("reactions are not used by these tests")
    }
    async fn upload_file_bytes(
        &self,
        _: &str,
        _: Vec<u8>,
    ) -> Result<ZulipUploadFileResponse, ZulipClientError> {
        panic!("uploads are not used by these tests")
    }
}

#[tokio::test]
async fn direct_command_uses_user_id_recipient_payload_when_all_recipients_are_numeric() {
    let transport = RecordingTransport::default();
    let command = ZulipExecutableCommand::new(
        "zulip-direct-user-id",
        "send_direct_message",
        None,
        json!({"recipients": ["101", "202"], "content": "Direct by user id"}),
    );
    let outcome = execute_zulip_command(&transport, &command)
        .await
        .expect("execute direct user id command");
    assert_eq!(outcome.result_payload["provider_message_id"], json!(9901));
    assert_eq!(
        transport
            .direct_user_ids
            .lock()
            .expect("direct user id calls")
            .as_slice(),
        &[vec![101, 202]]
    );
    assert!(
        transport
            .direct_recipients
            .lock()
            .expect("direct recipient calls")
            .is_empty()
    );
}

#[tokio::test]
async fn direct_command_keeps_email_recipient_payload_for_non_numeric_recipients() {
    let transport = RecordingTransport::default();
    let command = ZulipExecutableCommand::new(
        "zulip-direct-email",
        "send_direct_message",
        None,
        json!({"recipients": ["alice@example.test"], "content": "Direct by email"}),
    );
    let outcome = execute_zulip_command(&transport, &command)
        .await
        .expect("execute direct email command");
    assert_eq!(outcome.result_payload["provider_message_id"], json!(8801));
    assert_eq!(
        transport
            .direct_recipients
            .lock()
            .expect("direct recipient calls")
            .as_slice(),
        &[vec!["alice@example.test".to_owned()]]
    );
    assert!(
        transport
            .direct_user_ids
            .lock()
            .expect("direct user id calls")
            .is_empty()
    );
}
