use std::time::Duration;

use makosh_communications_api::accounts::CommunicationProviderKind;

use super::errors::EmailProviderNetworkError;
use super::helpers::validate_non_empty;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailFetchOptions {
    pub(super) max_results: u16,
    pub(super) query: Option<String>,
    pub(super) page_token: Option<String>,
    pub(super) label_ids: Vec<String>,
    pub(super) include_spam_trash: bool,
}

impl GmailFetchOptions {
    pub fn new(max_results: u16) -> Self {
        Self {
            max_results,
            query: None,
            page_token: None,
            label_ids: Vec::new(),
            include_spam_trash: true,
        }
    }

    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    pub fn page_token(mut self, page_token: impl Into<String>) -> Self {
        self.page_token = Some(page_token.into());
        self
    }

    pub fn label_id(mut self, label_id: impl Into<String>) -> Self {
        self.label_ids.push(label_id.into());
        self
    }

    pub fn include_spam_trash(mut self, include_spam_trash: bool) -> Self {
        self.include_spam_trash = include_spam_trash;
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        if self.max_results == 0 || self.max_results > 500 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_results",
                message: "must be between 1 and 500",
            });
        }
        if let Some(query) = &self.query {
            validate_non_empty("query", query)?;
        }
        if let Some(page_token) = &self.page_token {
            validate_non_empty("page_token", page_token)?;
        }
        for label_id in &self.label_ids {
            validate_non_empty("label_id", label_id)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailHistoryFetchOptions {
    pub(super) start_history_id: String,
    pub(super) max_results: u16,
    pub(super) page_token: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GmailContactFetchOptions {
    pub(super) page_size: u16,
    pub(super) page_token: Option<String>,
}

impl GmailContactFetchOptions {
    pub fn new(page_size: u16) -> Self {
        Self {
            page_size,
            page_token: None,
        }
    }

    pub fn page_token(mut self, page_token: impl Into<String>) -> Self {
        self.page_token = Some(page_token.into());
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        if self.page_size == 0 || self.page_size > 1000 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "page_size",
                message: "must be between 1 and 1000",
            });
        }
        if let Some(page_token) = &self.page_token {
            validate_non_empty("page_token", page_token)?;
        }

        Ok(())
    }
}

impl GmailHistoryFetchOptions {
    pub fn new(start_history_id: impl Into<String>, max_results: u16) -> Self {
        Self {
            start_history_id: start_history_id.into(),
            max_results,
            page_token: None,
        }
    }

    pub fn page_token(mut self, page_token: impl Into<String>) -> Self {
        self.page_token = Some(page_token.into());
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("start_history_id", &self.start_history_id)?;
        if self.max_results == 0 || self.max_results > 500 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_results",
                message: "must be between 1 and 500",
            });
        }
        if let Some(page_token) = &self.page_token {
            validate_non_empty("page_token", page_token)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapFetchOptions {
    pub provider_kind: CommunicationProviderKind,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub last_seen_uid: Option<u32>,
    pub max_messages: usize,
    pub latest_messages: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapMailboxListOptions {
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub username: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImapIdleOptions {
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub mailbox: String,
    pub username: String,
    pub timeout: Duration,
}

impl ImapFetchOptions {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        tls: bool,
        mailbox: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        Self {
            provider_kind: CommunicationProviderKind::Imap,
            host: host.into(),
            port,
            tls,
            mailbox: mailbox.into(),
            username: username.into(),
            last_seen_uid: None,
            max_messages: 100,
            latest_messages: false,
        }
    }

    pub fn provider_kind(mut self, provider_kind: CommunicationProviderKind) -> Self {
        self.provider_kind = provider_kind;
        self
    }

    pub fn last_seen_uid(mut self, last_seen_uid: u32) -> Self {
        self.last_seen_uid = Some(last_seen_uid);
        self
    }

    pub fn max_messages(mut self, max_messages: usize) -> Self {
        self.max_messages = max_messages;
        self
    }

    pub fn latest_messages(mut self) -> Self {
        self.latest_messages = true;
        self
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("host", &self.host)?;
        validate_non_empty("mailbox", &self.mailbox)?;
        validate_non_empty("username", &self.username)?;
        if self.port == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }
        if self.max_messages == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "max_messages",
                message: "must be greater than zero",
            });
        }

        Ok(())
    }
}

impl ImapMailboxListOptions {
    pub fn new(host: impl Into<String>, port: u16, tls: bool, username: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            port,
            tls,
            username: username.into(),
        }
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("host", &self.host)?;
        validate_non_empty("username", &self.username)?;
        if self.port == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }

        Ok(())
    }
}

impl ImapIdleOptions {
    pub fn new(
        host: impl Into<String>,
        port: u16,
        tls: bool,
        mailbox: impl Into<String>,
        username: impl Into<String>,
        timeout: Duration,
    ) -> Self {
        Self {
            host: host.into(),
            port,
            tls,
            mailbox: mailbox.into(),
            username: username.into(),
            timeout,
        }
    }

    pub(super) fn validate(&self) -> Result<(), EmailProviderNetworkError> {
        validate_non_empty("host", &self.host)?;
        validate_non_empty("mailbox", &self.mailbox)?;
        validate_non_empty("username", &self.username)?;
        if self.port == 0 {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "port",
                message: "must be greater than zero",
            });
        }
        if self.timeout.is_zero() || self.timeout > Duration::from_secs(29 * 60) {
            return Err(EmailProviderNetworkError::InvalidProviderRequest {
                field: "timeout",
                message: "must be between one nanosecond and 29 minutes",
            });
        }
        Ok(())
    }
}
