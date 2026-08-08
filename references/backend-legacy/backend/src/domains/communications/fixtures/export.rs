mod body;
mod encoded_words;
mod encoding;
pub mod errors;
mod headers;
pub mod models;
mod raw_payload;
mod redaction;
mod rfc822;
mod text;

use crate::domains::communications::sources::FixtureCommunicationSourceMessage;
use makosh_communications_api::email_sync::EmailSyncBatch;

use self::errors::EmailFixtureExportError;
use self::models::{EmailFixtureExportOptions, EmailFixturePrivacyMode};
use self::raw_payload::raw_rfc822_bytes;
use self::redaction::redact_message;
use self::rfc822::parse_rfc822_message;

pub fn export_fixture_messages_from_sync_batch(
    batch: &EmailSyncBatch,
    options: EmailFixtureExportOptions,
) -> Result<Vec<FixtureCommunicationSourceMessage>, EmailFixtureExportError> {
    batch
        .messages
        .iter()
        .map(|message| {
            let raw = raw_rfc822_bytes(&message.payload)?;
            let parsed = parse_rfc822_message(&raw)?;
            let parsed = match options.privacy_mode {
                EmailFixturePrivacyMode::Redacted => redact_message(
                    &message.provider_record_id,
                    &message.source_fingerprint,
                    message.occurred_at,
                    parsed,
                ),
            };

            Ok(FixtureCommunicationSourceMessage {
                provider_record_id: message.provider_record_id.clone(),
                subject: parsed.subject,
                from: parsed.from,
                to: parsed.to,
                sent_at: message.occurred_at,
                body_text: parsed.body_text,
                source_fingerprint: message.source_fingerprint.clone(),
            })
        })
        .collect()
}
