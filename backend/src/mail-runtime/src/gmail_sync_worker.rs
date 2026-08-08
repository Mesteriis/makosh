//! Bounded Gmail network execution outside the Mail control loop.

use makosh_mail_gmail::{
    GmailAdapterErrorV1, GmailApiClientV1, GmailListMessagesRequestV1, GmailRawMessageV1,
    history_message_ids,
};
use tokio::sync::{mpsc, oneshot};
use zeroize::Zeroizing;

pub struct PreparedGmailSyncProviderOperationV1 {
    pub(crate) connection_id: String,
    pub(crate) operation_id: String,
    pub(crate) client: GmailApiClientV1,
    pub(crate) access_token: Zeroizing<Vec<u8>>,
    pub(crate) cursor: GmailSyncProviderCursorV1,
    pub(crate) max_results: u16,
    pub(crate) windows: u32,
    pub(crate) observed_at_unix_seconds: i64,
    pub(crate) observed_at_nanos: i32,
    pub(crate) deadline_at_unix_seconds: i64,
}

impl PreparedGmailSyncProviderOperationV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }

    #[must_use]
    pub fn operation_id(&self) -> &str {
        &self.operation_id
    }

    #[must_use]
    pub const fn deadline_at_unix_seconds(&self) -> i64 {
        self.deadline_at_unix_seconds
    }
}

pub(crate) enum GmailSyncProviderCursorV1 {
    Full {
        page_token: Option<String>,
    },
    History {
        start_history_id: String,
        page_token: Option<String>,
    },
}

pub struct CompletedGmailSyncProviderOperationV1 {
    pub(crate) connection_id: String,
    pub(crate) operation_id: String,
    pub(crate) observed_messages: usize,
    pub(crate) outcome: GmailSyncProviderOutcomeV1,
    pub(crate) deadline_at_unix_seconds: i64,
}

impl CompletedGmailSyncProviderOperationV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

pub struct GmailSyncProviderPageDeliveryV1 {
    pub(crate) connection_id: String,
    pub(crate) operation_id: String,
    pub(crate) page: GmailSyncProviderPageV1,
    pub(crate) observed_at_unix_seconds: i64,
    pub(crate) observed_at_nanos: i32,
    pub(crate) acknowledgment: oneshot::Sender<bool>,
}

impl GmailSyncProviderPageDeliveryV1 {
    #[must_use]
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }
}

pub(crate) enum GmailSyncProviderPageV1 {
    Full {
        messages: Vec<(String, GmailRawMessageV1)>,
        next_page_token: Option<String>,
    },
    History {
        messages: Vec<(String, GmailRawMessageV1)>,
        start_history_id: String,
        checkpoint_history_id: String,
        next_page_token: Option<String>,
    },
}

pub(crate) enum GmailSyncProviderOutcomeV1 {
    Complete,
    HistoryExpired,
    Failed(GmailSyncProviderFailureV1),
}

pub(crate) enum GmailSyncProviderFailureV1 {
    Credential,
    Provider,
    Finalization,
}

pub async fn execute_gmail_sync_provider_operation(
    prepared: PreparedGmailSyncProviderOperationV1,
    page_sender: mpsc::Sender<GmailSyncProviderPageDeliveryV1>,
) -> CompletedGmailSyncProviderOperationV1 {
    let PreparedGmailSyncProviderOperationV1 {
        connection_id,
        operation_id,
        client,
        access_token,
        cursor,
        max_results,
        windows,
        observed_at_unix_seconds,
        observed_at_nanos,
        deadline_at_unix_seconds,
    } = prepared;
    let token = match std::str::from_utf8(&access_token) {
        Ok(token) => token,
        Err(_) => {
            return CompletedGmailSyncProviderOperationV1 {
                connection_id,
                operation_id,
                observed_messages: 0,
                outcome: GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Credential),
                deadline_at_unix_seconds,
            };
        }
    };
    let delivery = GmailSyncProviderPageDeliveryContextV1 {
        connection_id: &connection_id,
        operation_id: &operation_id,
        observed_at_unix_seconds,
        observed_at_nanos,
        page_sender: &page_sender,
    };
    let (observed_messages, outcome) = match cursor {
        GmailSyncProviderCursorV1::Full { page_token } => {
            fetch_full_pages(&client, token, page_token, max_results, windows, &delivery).await
        }
        GmailSyncProviderCursorV1::History {
            start_history_id,
            page_token,
        } => {
            fetch_history_pages(
                &client,
                token,
                start_history_id,
                page_token,
                windows,
                &delivery,
            )
            .await
        }
    };
    CompletedGmailSyncProviderOperationV1 {
        connection_id,
        operation_id,
        observed_messages,
        outcome,
        deadline_at_unix_seconds,
    }
}

struct GmailSyncProviderPageDeliveryContextV1<'a> {
    connection_id: &'a str,
    operation_id: &'a str,
    observed_at_unix_seconds: i64,
    observed_at_nanos: i32,
    page_sender: &'a mpsc::Sender<GmailSyncProviderPageDeliveryV1>,
}

async fn fetch_full_pages(
    client: &GmailApiClientV1,
    token: &str,
    mut page_token: Option<String>,
    max_results: u16,
    windows: u32,
    delivery: &GmailSyncProviderPageDeliveryContextV1<'_>,
) -> (usize, GmailSyncProviderOutcomeV1) {
    let mut observed_messages = 0_usize;
    for _ in 0..windows {
        let page = match client
            .list_messages(
                token,
                &GmailListMessagesRequestV1 {
                    max_results,
                    page_token,
                    query: None,
                    label_ids: Vec::new(),
                },
            )
            .await
        {
            Ok(page) => page,
            Err(_) => {
                return (
                    observed_messages,
                    GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider),
                );
            }
        };
        let next_page_token = page.next_page_token;
        let messages = match fetch_raw_messages(
            client,
            token,
            page.messages.into_iter().map(|message| message.id),
        )
        .await
        {
            Ok(messages) => messages,
            Err(error) => return (observed_messages, error),
        };
        let has_next_page = next_page_token.is_some();
        let page_messages = messages.len();
        if !deliver_page(
            delivery,
            GmailSyncProviderPageV1::Full {
                messages,
                next_page_token: next_page_token.clone(),
            },
        )
        .await
        {
            return (
                observed_messages,
                GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Finalization),
            );
        }
        observed_messages = observed_messages.saturating_add(page_messages);
        page_token = next_page_token;
        if !has_next_page {
            break;
        }
    }
    (observed_messages, GmailSyncProviderOutcomeV1::Complete)
}

async fn fetch_history_pages(
    client: &GmailApiClientV1,
    token: &str,
    start_history_id: String,
    mut page_token: Option<String>,
    windows: u32,
    delivery: &GmailSyncProviderPageDeliveryContextV1<'_>,
) -> (usize, GmailSyncProviderOutcomeV1) {
    let mut observed_messages = 0_usize;
    for _ in 0..windows {
        let page = match client
            .list_history(token, &start_history_id, page_token.as_deref())
            .await
        {
            Ok(page) => page,
            Err(GmailAdapterErrorV1::ProviderStatus(404)) => {
                return (
                    observed_messages,
                    GmailSyncProviderOutcomeV1::HistoryExpired,
                );
            }
            Err(_) => {
                return (
                    observed_messages,
                    GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider),
                );
            }
        };
        let message_ids = history_message_ids(&page);
        let Some(checkpoint_history_id) = page
            .history_id
            .as_deref()
            .filter(|value| valid_history_id(value))
            .map(str::to_owned)
        else {
            return (
                observed_messages,
                GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider),
            );
        };
        let messages = match fetch_raw_messages(client, token, message_ids.into_iter()).await {
            Ok(messages) => messages,
            Err(error) => return (observed_messages, error),
        };
        let next_page_token = page.next_page_token;
        let has_next_page = next_page_token.is_some();
        let page_messages = messages.len();
        if !deliver_page(
            delivery,
            GmailSyncProviderPageV1::History {
                messages,
                start_history_id: start_history_id.clone(),
                checkpoint_history_id,
                next_page_token: next_page_token.clone(),
            },
        )
        .await
        {
            return (
                observed_messages,
                GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Finalization),
            );
        }
        observed_messages = observed_messages.saturating_add(page_messages);
        page_token = next_page_token;
        if !has_next_page {
            break;
        }
    }
    (observed_messages, GmailSyncProviderOutcomeV1::Complete)
}

async fn deliver_page(
    delivery: &GmailSyncProviderPageDeliveryContextV1<'_>,
    page: GmailSyncProviderPageV1,
) -> bool {
    let (acknowledgment, committed) = oneshot::channel();
    if delivery
        .page_sender
        .send(GmailSyncProviderPageDeliveryV1 {
            connection_id: delivery.connection_id.to_owned(),
            operation_id: delivery.operation_id.to_owned(),
            page,
            observed_at_unix_seconds: delivery.observed_at_unix_seconds,
            observed_at_nanos: delivery.observed_at_nanos,
            acknowledgment,
        })
        .await
        .is_err()
    {
        return false;
    }
    matches!(committed.await, Ok(true))
}

async fn fetch_raw_messages(
    client: &GmailApiClientV1,
    token: &str,
    message_ids: impl Iterator<Item = String>,
) -> Result<Vec<(String, GmailRawMessageV1)>, GmailSyncProviderOutcomeV1> {
    let mut messages = Vec::new();
    for message_id in message_ids {
        let raw = client
            .fetch_raw_message(token, &message_id)
            .await
            .map_err(|_| {
                GmailSyncProviderOutcomeV1::Failed(GmailSyncProviderFailureV1::Provider)
            })?;
        messages.push((message_id, raw));
    }
    Ok(messages)
}

fn valid_history_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::{
        GmailSyncProviderPageDeliveryContextV1, GmailSyncProviderPageV1, deliver_page,
        valid_history_id,
    };

    #[test]
    fn history_cursor_accepts_only_nonempty_decimal_ids() {
        assert!(valid_history_id("123"));
        assert!(!valid_history_id(""));
        assert!(!valid_history_id("12a"));
        assert!(!valid_history_id("-1"));
    }

    #[tokio::test]
    async fn page_delivery_waits_for_mail_finalization_acknowledgment() {
        let (page_sender, mut pages) = tokio::sync::mpsc::channel(1);
        let context = GmailSyncProviderPageDeliveryContextV1 {
            connection_id: "mail-account-1",
            operation_id: "sync-operation-1",
            observed_at_unix_seconds: 42,
            observed_at_nanos: 7,
            page_sender: &page_sender,
        };
        let mut delivery = Box::pin(deliver_page(
            &context,
            GmailSyncProviderPageV1::Full {
                messages: Vec::new(),
                next_page_token: None,
            },
        ));
        let page = tokio::select! {
            result = &mut delivery => panic!("page completed before acknowledgment: {result}"),
            page = pages.recv() => page.expect("bounded Gmail page delivery"),
        };

        assert_eq!(page.connection_id, "mail-account-1");
        assert_eq!(page.operation_id, "sync-operation-1");
        page.acknowledgment
            .send(true)
            .expect("acknowledge Gmail page");
        assert!(delivery.await);
    }
}
