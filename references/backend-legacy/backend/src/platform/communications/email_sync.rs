use serde_json::Value;
use thiserror::Error;

use super::ProviderAccount;
use makosh_communications_api::accounts::{
    CommunicationProviderKind, ProviderAccountSecretPurpose,
};
use makosh_communications_api::email_sync::{EmailSyncAdapterConfig, EmailSyncPlan};

const DEFAULT_IMAP_MAILBOX: &str = "INBOX";
pub const IMAP_ALL_MAILBOXES: &str = "*";
const IMAP_STREAM_PREFIX: &str = "imap:";

#[derive(Debug, Error)]
pub enum EmailSyncPlanError {
    #[error("invalid provider config field {field}: {message}")]
    InvalidProviderConfig {
        field: &'static str,
        message: &'static str,
    },

    #[error("provider account config must not contain secret-like key: {key}")]
    SecretLikeConfigKey { key: String },
}

pub fn plan_email_sync(account: &ProviderAccount) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let account_id = validate_non_empty("account_id", &account.account_id)?;
    reject_secret_like_config_keys(&account.config)?;

    match account.provider_kind {
        CommunicationProviderKind::Gmail => plan_gmail_sync(account, account_id),
        CommunicationProviderKind::Icloud | CommunicationProviderKind::Imap => {
            plan_imap_sync(account, account_id)
        }
        CommunicationProviderKind::TelegramUser
        | CommunicationProviderKind::TelegramBot
        | CommunicationProviderKind::WhatsappWeb
        | CommunicationProviderKind::WhatsappBusinessCloud
        | CommunicationProviderKind::ZulipBot
        | CommunicationProviderKind::ZoomUser
        | CommunicationProviderKind::ZoomServerToServer
        | CommunicationProviderKind::YandexTelemostUser => {
            Err(EmailSyncPlanError::InvalidProviderConfig {
                field: "provider_kind",
                message: "email sync supports only gmail, icloud or imap",
            })
        }
    }
}

pub fn imap_mailbox_stream_id(mailbox: &str) -> String {
    let mut stream_id = String::from(IMAP_STREAM_PREFIX);

    for character in mailbox.chars() {
        match character {
            '%' => stream_id.push_str("%25"),
            ':' => stream_id.push_str("%3A"),
            _ => stream_id.push(character),
        }
    }

    stream_id
}

pub fn email_sync_plan_stream_ids(plan: &EmailSyncPlan) -> Vec<String> {
    match &plan.adapter_config {
        EmailSyncAdapterConfig::Gmail { .. } => vec![plan.stream_id.clone()],
        EmailSyncAdapterConfig::Imap { mailboxes, .. } => mailboxes
            .iter()
            .filter(|mailbox| !imap_mailbox_selects_all(mailbox))
            .map(|mailbox| imap_mailbox_stream_id(mailbox))
            .collect(),
    }
}

pub fn email_sync_plan_selects_all_imap_mailboxes(plan: &EmailSyncPlan) -> bool {
    matches!(
        &plan.adapter_config,
        EmailSyncAdapterConfig::Imap { mailboxes, .. }
            if mailboxes.iter().any(|mailbox| imap_mailbox_selects_all(mailbox))
    )
}

pub fn imap_mailbox_stream_prefix() -> &'static str {
    IMAP_STREAM_PREFIX
}

fn plan_gmail_sync(
    account: &ProviderAccount,
    account_id: String,
) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let history_stream_id = optional_string(&account.config, "history_stream_id")?
        .unwrap_or_else(|| "gmail:history".to_owned());
    validate_non_empty("history_stream_id", &history_stream_id)?;
    validate_no_control_chars("history_stream_id", &history_stream_id)?;

    Ok(EmailSyncPlan {
        account_id,
        provider_kind: account.provider_kind,
        credential_purpose: ProviderAccountSecretPurpose::OauthToken,
        stream_id: history_stream_id.clone(),
        adapter_config: EmailSyncAdapterConfig::Gmail { history_stream_id },
    })
}

fn plan_imap_sync(
    account: &ProviderAccount,
    account_id: String,
) -> Result<EmailSyncPlan, EmailSyncPlanError> {
    let host = required_string(&account.config, "host")?;
    let port = required_port(&account.config, "port")?;
    let tls = required_bool(&account.config, "tls")?;
    let mailboxes = imap_mailboxes(&account.config)?;
    let stream_id = imap_mailbox_stream_id(
        mailboxes
            .first()
            .map(String::as_str)
            .unwrap_or(DEFAULT_IMAP_MAILBOX),
    );

    Ok(EmailSyncPlan {
        account_id,
        provider_kind: account.provider_kind,
        credential_purpose: ProviderAccountSecretPurpose::ImapPassword,
        stream_id,
        adapter_config: EmailSyncAdapterConfig::Imap {
            host,
            port,
            tls,
            mailboxes,
        },
    })
}

fn imap_mailboxes(config: &Value) -> Result<Vec<String>, EmailSyncPlanError> {
    if optional_bool(config, "sync_all_mailboxes")?.unwrap_or(false) {
        return Ok(vec![IMAP_ALL_MAILBOXES.to_owned()]);
    }

    if let Some(mailboxes) = optional_string_array(config, "mailboxes")? {
        return validate_mailboxes(mailboxes);
    }

    let mailbox =
        optional_string(config, "mailbox")?.unwrap_or_else(|| DEFAULT_IMAP_MAILBOX.to_owned());
    validate_mailboxes(vec![mailbox])
}

fn validate_mailboxes(mailboxes: Vec<String>) -> Result<Vec<String>, EmailSyncPlanError> {
    let mut validated = Vec::new();
    for mailbox in mailboxes {
        let mailbox = validate_non_empty("mailbox", &mailbox)?;
        validate_no_control_chars("mailbox", &mailbox)?;
        if imap_mailbox_selects_all(&mailbox) {
            return Ok(vec![IMAP_ALL_MAILBOXES.to_owned()]);
        }
        if !validated.contains(&mailbox) {
            validated.push(mailbox);
        }
    }
    if validated.is_empty() {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field: "mailboxes",
            message: "must contain at least one mailbox",
        });
    }

    Ok(validated)
}

fn imap_mailbox_selects_all(mailbox: &str) -> bool {
    matches!(mailbox.trim(), IMAP_ALL_MAILBOXES)
}

fn required_string(config: &Value, field: &'static str) -> Result<String, EmailSyncPlanError> {
    let Some(value) = optional_string(config, field)? else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "missing string value",
        });
    };
    validate_non_empty(field, &value)
}

fn optional_string(
    config: &Value,
    field: &'static str,
) -> Result<Option<String>, EmailSyncPlanError> {
    let Some(value) = config.get(field) else {
        return Ok(None);
    };
    let Some(value) = value.as_str() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected string value",
        });
    };

    Ok(Some(value.trim().to_owned()))
}

fn optional_string_array(
    config: &Value,
    field: &'static str,
) -> Result<Option<Vec<String>>, EmailSyncPlanError> {
    let Some(value) = config.get(field) else {
        return Ok(None);
    };
    let Some(values) = value.as_array() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected string array",
        });
    };

    let mut strings = Vec::with_capacity(values.len());
    for value in values {
        let Some(value) = value.as_str() else {
            return Err(EmailSyncPlanError::InvalidProviderConfig {
                field,
                message: "expected string array",
            });
        };
        strings.push(value.trim().to_owned());
    }

    Ok(Some(strings))
}

fn required_port(config: &Value, field: &'static str) -> Result<u16, EmailSyncPlanError> {
    let Some(value) = config.get(field).and_then(Value::as_u64) else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected integer port",
        });
    };
    let Ok(port) = u16::try_from(value) else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "port must fit u16",
        });
    };
    if port == 0 {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "port must be greater than zero",
        });
    }

    Ok(port)
}

fn required_bool(config: &Value, field: &'static str) -> Result<bool, EmailSyncPlanError> {
    config
        .get(field)
        .and_then(Value::as_bool)
        .ok_or(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected boolean value",
        })
}

fn optional_bool(config: &Value, field: &'static str) -> Result<Option<bool>, EmailSyncPlanError> {
    let Some(value) = config.get(field) else {
        return Ok(None);
    };
    let Some(value) = value.as_bool() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "expected boolean value",
        });
    };

    Ok(Some(value))
}

fn reject_secret_like_config_keys(config: &Value) -> Result<(), EmailSyncPlanError> {
    let Some(object) = config.as_object() else {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field: "config",
            message: "expected object",
        });
    };

    for (key, value) in object {
        let key_path = key.clone();
        reject_secret_like_config_key(key, &key_path)?;
        reject_secret_like_config_value(value, &key_path)?;
    }

    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<String, EmailSyncPlanError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "must not be empty",
        });
    }

    Ok(value.to_owned())
}

fn validate_no_control_chars(field: &'static str, value: &str) -> Result<(), EmailSyncPlanError> {
    if value.chars().any(char::is_control) {
        return Err(EmailSyncPlanError::InvalidProviderConfig {
            field,
            message: "must not contain control characters",
        });
    }

    Ok(())
}

fn reject_secret_like_config_value(value: &Value, path: &str) -> Result<(), EmailSyncPlanError> {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                let key_path = format!("{path}.{key}");
                reject_secret_like_config_key(key, &key_path)?;
                reject_secret_like_config_value(value, &key_path)?;
            }
            Ok(())
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                reject_secret_like_config_value(value, &format!("{path}[{index}]"))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

fn reject_secret_like_config_key(key: &str, key_path: &str) -> Result<(), EmailSyncPlanError> {
    let normalized = key.to_ascii_lowercase();
    if normalized.contains("password")
        || normalized.contains("secret")
        || normalized.contains("token")
        || normalized.contains("credential")
    {
        return Err(EmailSyncPlanError::SecretLikeConfigKey {
            key: key_path.to_owned(),
        });
    }

    Ok(())
}
