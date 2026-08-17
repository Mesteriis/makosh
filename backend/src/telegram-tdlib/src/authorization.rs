//! Long-lived TDLib authorization driver. Business lifecycle remains in telegram-core.

use std::{fmt, time::Duration};

use serde_json::{Value, json};

use crate::{
    TdJsonClient, TdlibAuthorizationParameters, TdlibAuthorizationUpdate, TdlibError,
    check_authentication_password, check_database_encryption_key_request, close_session_request,
    parse_authorization_update, parse_qr_authorization_link, request_qr_code_authentication,
    set_tdlib_parameters_request,
};

#[derive(Clone, Eq, PartialEq)]
pub enum TdlibAuthorizationEvent {
    State(TdlibAuthorizationUpdate),
    QrLink(String),
}

impl fmt::Debug for TdlibAuthorizationEvent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(state) => formatter.debug_tuple("State").field(state).finish(),
            Self::QrLink(_) => formatter
                .debug_tuple("QrLink")
                .field(&"[redacted]")
                .finish(),
        }
    }
}

pub trait AuthorizationClient {
    fn send_json(&self, request: &Value) -> Result<(), TdlibError>;
    fn receive_json(&self, timeout_seconds: f64) -> Result<Option<Value>, TdlibError>;
}

impl AuthorizationClient for TdJsonClient {
    fn send_json(&self, request: &Value) -> Result<(), TdlibError> {
        TdJsonClient::send_json(self, request)
    }

    fn receive_json(&self, timeout_seconds: f64) -> Result<Option<Value>, TdlibError> {
        TdJsonClient::receive_json(self, timeout_seconds)
    }
}

pub struct TdlibAuthorizationDriver<C = TdJsonClient> {
    client: C,
    parameters: TdlibAuthorizationParameters,
    parameters_sent: bool,
    encryption_key_checked: bool,
    qr_requested: bool,
}

impl<C> TdlibAuthorizationDriver<C>
where
    C: AuthorizationClient,
{
    pub fn new(client: C, parameters: TdlibAuthorizationParameters) -> Result<Self, TdlibError> {
        client.send_json(&json!({
            "@type": "getAuthorizationState",
            "@extra": "makosh-initial-authorization-state"
        }))?;
        Ok(Self {
            client,
            parameters,
            parameters_sent: false,
            encryption_key_checked: false,
            qr_requested: false,
        })
    }

    pub fn initialize(&mut self) -> Result<(), TdlibError> {
        self.client
            .send_json(&set_tdlib_parameters_request(&self.parameters)?)?;
        self.parameters_sent = true;
        Ok(())
    }

    pub fn poll(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<TdlibAuthorizationEvent>, TdlibError> {
        let timeout_seconds = timeout.as_secs_f64();
        let Some(payload) = self.client.receive_json(timeout_seconds)? else {
            return Ok(None);
        };
        if !is_authorization_payload(&payload) {
            return Ok(None);
        }
        self.handle_payload(payload).map(Some)
    }

    pub fn handle_payload(
        &mut self,
        payload: Value,
    ) -> Result<TdlibAuthorizationEvent, TdlibError> {
        let update = parse_authorization_update(&payload)?;
        match &update {
            TdlibAuthorizationUpdate::WaitingParameters if !self.parameters_sent => {
                self.initialize()?;
            }
            TdlibAuthorizationUpdate::WaitingEncryptionKey if !self.encryption_key_checked => {
                self.client
                    .send_json(&check_database_encryption_key_request(
                        self.parameters
                            .session_encryption_key
                            .as_deref()
                            .map(|value| value.as_slice()),
                    ))?;
                self.encryption_key_checked = true;
            }
            TdlibAuthorizationUpdate::Other(state)
                if matches!(
                    state.as_str(),
                    "authorizationStateWaitPhoneNumber" | "authorizationStateWaitCode"
                ) && !self.qr_requested =>
            {
                self.client.send_json(&request_qr_code_authentication())?;
                self.qr_requested = true;
            }
            TdlibAuthorizationUpdate::WaitingQrScan => {
                return Ok(TdlibAuthorizationEvent::QrLink(
                    parse_qr_authorization_link(&payload)?,
                ));
            }
            _ => {}
        }
        Ok(TdlibAuthorizationEvent::State(update))
    }

    pub fn submit_password(&self, password: &str) -> Result<(), TdlibError> {
        self.client
            .send_json(&check_authentication_password(password)?)
    }

    pub fn close(&self) -> Result<(), TdlibError> {
        self.client.send_json(&close_session_request())
    }

    pub fn into_client(self) -> C {
        self.client
    }
}

impl TdlibAuthorizationDriver<TdJsonClient> {
    pub fn into_transport(
        self,
        account_id: impl Into<String>,
    ) -> Result<crate::TdJsonTransport, TdlibError> {
        crate::TdJsonTransport::new(self.client, account_id)
    }
}

fn is_authorization_payload(payload: &Value) -> bool {
    matches!(
        payload.get("@type").and_then(Value::as_str),
        Some("updateAuthorizationState" | "error")
    ) || payload
        .get("@type")
        .and_then(Value::as_str)
        .is_some_and(|kind| kind.starts_with("authorizationState"))
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::VecDeque, path::PathBuf};

    use serde_json::Value;
    use serde_json::json;
    use zeroize::Zeroizing;

    use super::{AuthorizationClient, TdlibAuthorizationDriver, is_authorization_payload};
    use crate::{TdlibAuthorizationParameters, TdlibError};

    #[derive(Default)]
    struct RecordingAuthorizationClient {
        sent: RefCell<Vec<Value>>,
        received: RefCell<VecDeque<Value>>,
    }

    impl AuthorizationClient for RecordingAuthorizationClient {
        fn send_json(&self, request: &Value) -> Result<(), TdlibError> {
            self.sent.borrow_mut().push(request.clone());
            Ok(())
        }

        fn receive_json(&self, _timeout_seconds: f64) -> Result<Option<Value>, TdlibError> {
            Ok(self.received.borrow_mut().pop_front())
        }
    }

    #[test]
    fn starts_by_requesting_the_current_authorization_state() {
        let client = RecordingAuthorizationClient::default();
        let driver = TdlibAuthorizationDriver::new(
            client,
            TdlibAuthorizationParameters {
                api_id: 1,
                api_hash: Zeroizing::new("hash".to_owned()),
                database_directory: PathBuf::from("database"),
                session_encryption_key: None,
            },
        )
        .expect("authorization driver");

        assert_eq!(
            driver.client.sent.into_inner(),
            vec![json!({
                "@type": "getAuthorizationState",
                "@extra": "makosh-initial-authorization-state"
            })]
        );
    }

    #[test]
    fn ignores_tdlib_acknowledgements_without_overwriting_authorization_state() {
        assert!(!is_authorization_payload(&json!({
            "@type": "ok",
            "@extra": "makosh-request-qr-code-authentication"
        })));
        assert!(!is_authorization_payload(&json!({
            "@type": "updateOption",
            "name": "version"
        })));

        assert!(is_authorization_payload(&json!({
            "@type": "updateAuthorizationState",
            "authorization_state": {
                "@type": "authorizationStateWaitOtherDeviceConfirmation",
                "link": "tg://redacted"
            }
        })));
        assert!(is_authorization_payload(&json!({
            "@type": "authorizationStateWaitPassword"
        })));
        assert!(is_authorization_payload(&json!({
            "@type": "error",
            "code": 400
        })));
    }
}
